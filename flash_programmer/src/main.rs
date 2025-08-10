#![no_std]
#![no_main]

mod programmer;
mod flash_buffer;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi as Stm32Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, mode};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as EmbassySpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use w25::{W25, Q, Error};
use programmer::FlashProgrammer;
use flash_buffer::{FlashBuffer, BufferStatus};
// RTT functionality removed - using defmt only
use {defmt_rtt as _, panic_probe as _};



// #[global_allocator]
// static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Irqs {
    // SPI2 => embassy_stm32::spi::InterruptHandler<peripherals::SPI2>;
});

/// Configure STM32 system
fn configure_stm32() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
    }

    // Enable dead battery support
    config.enable_ucpd1_dead_battery = true;

    config
}

use crate::programmer::DummyPin;

/// Initialize SPI2 for W25Q128 Flash communication
async fn initialize_flash_spi(p: embassy_stm32::Peripherals) -> W25<Q, EmbassySpiDevice<'static, CriticalSectionRawMutex, Stm32Spi<'static, mode::Async>, Output<'static>>, DummyPin, DummyPin> {
    info!("Initializing SPI2 for W25Q128 Flash...");

    // SPI2 pins for W25Q128 Flash
    let sck_pin = p.PB13;   // SPI2_SCK
    let mosi_pin = p.PB15;  // SPI2_MOSI
    let miso_pin = p.PB14;  // SPI2_MISO (corrected from PA10)
    let cs_pin_output = Output::new(p.PB12, Level::High, Speed::VeryHigh); // SPI2_NSS

    // Configure WP (Write Protect) pin - CRITICAL for Flash writing!
    let _wp_pin_output = Output::new(p.PB11, Level::High, Speed::VeryHigh); // WP must be HIGH to enable writing
    let wp_pin = DummyPin; // Still use DummyPin for the driver, but we control WP separately
    let hold_pin = DummyPin;

    info!("WP pin (PB11) configured as HIGH - Write protection DISABLED");

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000); // 1MHz for Flash communication (reduced for debugging)

    // W25Q128 requires SPI Mode 0 (CPOL=0, CPHA=0)
    // Default SPI config should be Mode 0, which is what we need
    info!("SPI Config - Frequency: {} Hz", spi_config.frequency.0);

    let spi_bus = Stm32Spi::new(
        p.SPI2,
        sck_pin,
        mosi_pin,
        miso_pin,
        p.DMA1_CH4, // TX DMA
        p.DMA1_CH5, // RX DMA
        spi_config,
    );

    static SPI_BUS_CELL: StaticCell<
        Mutex<CriticalSectionRawMutex, Stm32Spi<'static, mode::Async>>,
    > = StaticCell::new();
    let spi_bus_mutex_ref = SPI_BUS_CELL.init(Mutex::new(spi_bus));

    let spi_device = EmbassySpiDevice::<
        'static,
        CriticalSectionRawMutex,
        Stm32Spi<'static, mode::Async>,
        Output<'static>,
    >::new(spi_bus_mutex_ref, cs_pin_output);

    let flash = match W25::new(spi_device, hold_pin, wp_pin, 16 * 1024 * 1024) { // 128Mbit = 16MB
        Ok(flash) => {
            info!("W25Q128 Flash initialized successfully!");
            flash
        }
        Err(e) => {
            error!("Failed to initialize W25Q128 Flash: {:?}", e);
            core::panic!("Flash initialization failed");
        }
    };

    flash
}







#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting W25Q128 Flash Programmer with Buffer Protocol");

    // Configure STM32 system
    let config = configure_stm32();
    let p = embassy_stm32::init(config);

    // Initialize Flash
    let flash = initialize_flash_spi(p).await;
    let mut programmer = FlashProgrammer::new(flash);

    // Get device info
    info!("Reading device information...");
    match programmer.get_device_info().await {
        Ok(device_info) => {
            device_info.print_info();
        }
        Err(e) => {
            error!("Failed to read device info: {:?}", e);
            return;
        }
    }

    // Initialize flash buffer
    let flash_buffer = unsafe { FlashBuffer::new() };
    // Don't clear buffer - preserve any existing data
    info!("Flash buffer initialized");

    // Test Flash read operation
    info!("Testing Flash read operation...");
    let mut read_buffer = [0u8; 16];
    match programmer.read_data(0x000000, &mut read_buffer).await {
        Ok(()) => {
            info!("✓ Flash read test successful");
            info!("Data at address 0x000000: {:?}", read_buffer);
        }
        Err(e) => {
            error!("✗ Flash read test failed: {:?}", e);
        }
    }

    info!("Flash programmer ready - monitoring buffer for programming requests");

    // Main programming loop
    let mut data_buffer = [0u8; 2032]; // Maximum data size
    let mut total_programmed = 0u32;

    loop {
        // Check buffer status every 10ms
        Timer::after(Duration::from_millis(10)).await;

        let status = flash_buffer.read_status();

        // Check for verify request first
        if flash_buffer.has_verify_request() {
            // Handle verify request even if status doesn't match
            info!("Detected verify request with magic 0xCAFEBABE");
            let start_address = flash_buffer.read_address();
            let verify_length = flash_buffer.read_length();

            info!("Verifying {} bytes from address 0x{:08X}", verify_length, start_address);
            flash_buffer.write_status(BufferStatus::Programming);

            // 简化验证：只检查前1KB数据
            let verify_size = verify_length.min(1024) as usize;
            let mut verify_buffer = [0u8; 1024];

            match programmer.read_data(start_address, &mut verify_buffer[..verify_size]).await {
                Ok(()) => {
                    info!("✓ Verification successful - first {} bytes read OK", verify_size);
                    info!("Sample data: {:?}", &verify_buffer[..16.min(verify_size)]);
                    flash_buffer.write_status(BufferStatus::VerifyComplete);
                }
                Err(e) => {
                    error!("✗ Verification failed: {:?}", e);
                    flash_buffer.write_status(BufferStatus::VerifyError);
                }
            }
            continue;
        }

        match status {
            BufferStatus::HasData => {
                // New programming request received
                info!("Detected HasData status - processing request");

                // Debug: print buffer stats
                let stats = flash_buffer.get_stats();
                info!("Buffer stats: {:?}", stats);

                if let Some(request) = flash_buffer.get_request() {
                    info!("Programming request: {:?}", request);
                    flash_buffer.write_status(BufferStatus::Programming);

                    // Read data from buffer
                    match flash_buffer.read_data(&mut data_buffer[..request.length]) {
                        Ok(bytes_read) => {
                            info!("Read {} bytes from buffer", bytes_read);
                            info!("First 16 bytes: {:?}", &data_buffer[..bytes_read.min(16)]);

                            // Program data to flash
                            match programmer.program_data(request.address, &data_buffer[..bytes_read]).await {
                                Ok(()) => {
                                    total_programmed += bytes_read as u32;
                                    info!("✓ Programming successful. Total: {} bytes", total_programmed);
                                    flash_buffer.write_status(BufferStatus::Complete);
                                }
                                Err(e) => {
                                    error!("✗ Programming failed: {:?}", e);
                                    flash_buffer.write_status(BufferStatus::Error);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read data from buffer: {}", e);
                            flash_buffer.write_status(BufferStatus::Error);
                        }
                    }
                } else {
                    error!("Invalid data in buffer - magic or validation failed");
                    let stats = flash_buffer.get_stats();
                    error!("Buffer stats: {:?}", stats);
                    flash_buffer.write_status(BufferStatus::Error);
                }
            }
            BufferStatus::Idle => {
                // Normal state - no action needed
            }
            BufferStatus::Programming => {
                // Should not happen in this implementation
                warn!("Unexpected Programming status");
            }
            BufferStatus::VerifyRequest => {
                // Verification request received
                info!("Detected VerifyRequest - starting verification");

                let start_address = flash_buffer.read_address();
                let verify_length = flash_buffer.read_length();

                info!("Verifying {} bytes from address 0x{:08X}", verify_length, start_address);
                flash_buffer.write_status(BufferStatus::Programming);

                // 简化验证：只检查前1KB数据
                let verify_size = verify_length.min(1024) as usize;
                let mut verify_buffer = [0u8; 1024];

                match programmer.read_data(start_address, &mut verify_buffer[..verify_size]).await {
                    Ok(()) => {
                        info!("✓ Verification successful - first {} bytes read OK", verify_size);
                        info!("Sample data: {:?}", &verify_buffer[..16.min(verify_size)]);
                        flash_buffer.write_status(BufferStatus::VerifyComplete);
                    }
                    Err(e) => {
                        error!("✗ Verification failed: {:?}", e);
                        flash_buffer.write_status(BufferStatus::VerifyError);
                    }
                }
            }
            BufferStatus::Complete => {
                // Previous operation completed - wait for buffer to be cleared
            }
            BufferStatus::VerifyComplete => {
                // Previous verification completed - wait for buffer to be cleared
            }
            BufferStatus::Error | BufferStatus::VerifyError => {
                // Previous operation failed - wait for buffer to be cleared
            }
        }
    }
}
