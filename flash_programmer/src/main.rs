#![no_std]
#![no_main]

mod programmer;


use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi as Stm32Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, mode};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as EmbassySpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
// use embedded_alloc::Heap;
use static_cell::StaticCell;
use w25::{W25, Q, Error};
use programmer::FlashProgrammer;
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
    info!("Starting W25Q128 Flash Programmer");

    // Initialize the allocator (disabled for now)
    // {
    //     use core::mem::MaybeUninit;
    //     const HEAP_SIZE: usize = 8192;
    //     static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    //     unsafe { HEAP.init(ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    // }

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

    // Do NOT automatically erase chip to protect existing data
    info!("Flash chip ready - existing data preserved");

    // Test Flash read operation only (no erase/write to avoid blocking)
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

    info!("Flash programmer ready for external commands");

    info!("Flash programmer finished.");

    // Keep running for debugging
    loop {
        cortex_m::asm::wfi();
    }
}
