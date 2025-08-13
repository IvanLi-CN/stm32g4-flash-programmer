#![no_std]
#![no_main]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use flash_protocol::*;
use panic_halt as _;
use alloc::vec::Vec;

mod flash_driver;

use flash_driver::FlashDriver;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

type SharedSpi = Mutex<CriticalSectionRawMutex, Spi<'static, peripherals::SPI2>>;
type FlashSpiDevice = SpiDevice<'static, CriticalSectionRawMutex, Spi<'static, peripherals::SPI2>, Output<'static>>;

static SPI_BUS: Mutex<CriticalSectionRawMutex, Option<Spi<'static, peripherals::SPI2>>> = Mutex::new(None);

// Simple heap for dynamic allocation (8KB)
static mut HEAP: [u8; 8192] = [0; 8192];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP.len());
    }

    info!("STM32G4 Flash Programmer starting...");

    // Initialize hardware
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: Some(PllQDiv::DIV2), // 48 MHz for USB
            divr: Some(PllRDiv::DIV2), // 170 MHz for system
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV1;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    let p = embassy_stm32::init(config);
    info!("Hardware initialized");

    // Initialize SPI for Flash
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(10_000_000); // 10 MHz
    
    let spi = Spi::new(
        p.SPI2,
        p.PB13, // SCK
        p.PB15, // MOSI
        p.PB14, // MISO
        p.DMA1_CH1,
        p.DMA1_CH2,
        spi_config,
    );

    // Store SPI in global mutex
    *SPI_BUS.lock().await = Some(spi);

    // Create SPI device for Flash
    let cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);
    let flash_device = SpiDevice::new(&SPI_BUS, cs);

    // Initialize Flash driver
    let mut flash_driver = FlashDriver::new(flash_device);
    match flash_driver.init().await {
        Ok(_) => info!("Flash initialized successfully"),
        Err(e) => {
            error!("Flash initialization failed: {:?}", e);
            // Continue anyway for testing
        }
    }

    // Initialize USB
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    
    // Create embassy-usb Config
    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("STM32G4 Flash Programmer");
    usb_config.product = Some("Flash Programmer");
    usb_config.serial_number = Some("12345678");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    // Required for Windows compatibility
    usb_config.device_class = 0xEF;
    usb_config.device_sub_class = 0x02;
    usb_config.device_protocol = 0x01;
    usb_config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();
    let mut builder = Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create CDC-ACM class
    let mut cdc_class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut usb_device = builder.build();

    info!("USB initialized");

    // Spawn USB task
    spawner.spawn(usb_task(usb_device)).unwrap();

    // Spawn protocol handler task
    spawner.spawn(protocol_task(cdc_class, flash_driver)).unwrap();

    info!("All tasks spawned, entering main loop");

    // Main loop - just blink LED to show we're alive
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb_device: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    usb_device.run().await;
}

#[embassy_executor::task]
async fn protocol_task(
    mut cdc_class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
    mut flash_driver: FlashDriver<FlashSpiDevice>,
) {
    info!("Protocol handler starting...");
    
    // Wait a bit for USB to be ready
    Timer::after(Duration::from_secs(2)).await;
    
    let mut buffer = [0u8; 1024];
    let mut packet_buffer = Vec::new();
    
    loop {
        // Try to read data
        match cdc_class.read_packet(&mut buffer).await {
            Ok(n) if n > 0 => {
                info!("Received {} bytes", n);
                
                // Add to packet buffer
                packet_buffer.extend_from_slice(&buffer[..n]);
                
                // Try to parse a complete packet
                if let Ok(packet) = Packet::from_bytes(&packet_buffer) {
                    info!("Processing command: {:?}", packet.command);
                    
                    // Process the command
                    let response = match packet.command {
                        Command::Info => handle_info_command(&mut flash_driver).await,
                        Command::Erase => handle_erase_command(&packet, &mut flash_driver).await,
                        Command::Write => handle_write_command(&packet, &mut flash_driver).await,
                        Command::Read => handle_read_command(&packet, &mut flash_driver).await,
                        Command::Verify => handle_verify_command(&packet, &mut flash_driver).await,
                    };
                    
                    // Send response
                    let response_data = response.to_bytes();
                    if let Err(e) = cdc_class.write_packet(&response_data).await {
                        error!("Failed to send response: {:?}", e);
                    }
                    
                    // Clear packet buffer
                    packet_buffer.clear();
                }
            }
            Ok(_) => {
                // No data received, continue
            }
            Err(e) => {
                warn!("USB read error: {:?}", e);
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        
        Timer::after(Duration::from_millis(10)).await;
    }
}

async fn handle_info_command(flash_driver: &mut FlashDriver<FlashSpiDevice>) -> Response {
    match flash_driver.get_info().await {
        Ok(info) => {
            let mut data = Vec::new();
            data.extend_from_slice(&info.jedec_id.to_le_bytes());
            data.extend_from_slice(&info.total_size.to_le_bytes());
            data.extend_from_slice(&info.page_size.to_le_bytes());
            data.extend_from_slice(&info.sector_size.to_le_bytes());
            Response::new(Status::Success, data)
        }
        Err(_) => Response::new(Status::FlashError, Vec::new()),
    }
}

async fn handle_erase_command(packet: &Packet, flash_driver: &mut FlashDriver<FlashSpiDevice>) -> Response {
    let size = if packet.data.len() >= 4 {
        u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
    } else {
        FLASH_SECTOR_SIZE as u32
    };

    match flash_driver.erase_range(packet.address, size).await {
        Ok(_) => Response::new(Status::Success, Vec::new()),
        Err(_) => Response::new(Status::FlashError, Vec::new()),
    }
}

async fn handle_write_command(packet: &Packet, flash_driver: &mut FlashDriver<FlashSpiDevice>) -> Response {
    match flash_driver.write_data(packet.address, &packet.data).await {
        Ok(_) => Response::new(Status::Success, Vec::new()),
        Err(_) => Response::new(Status::FlashError, Vec::new()),
    }
}

async fn handle_read_command(packet: &Packet, flash_driver: &mut FlashDriver<FlashSpiDevice>) -> Response {
    let size = if packet.data.len() >= 4 {
        u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
    } else {
        return Response::new(Status::InvalidAddress, Vec::new());
    };

    let mut buffer = vec![0u8; size as usize];
    match flash_driver.read_data(packet.address, &mut buffer).await {
        Ok(_) => Response::new(Status::Success, buffer),
        Err(_) => Response::new(Status::FlashError, Vec::new()),
    }
}

async fn handle_verify_command(packet: &Packet, flash_driver: &mut FlashDriver<FlashSpiDevice>) -> Response {
    let mut buffer = vec![0u8; packet.data.len()];
    match flash_driver.read_data(packet.address, &mut buffer).await {
        Ok(_) => {
            if buffer == packet.data {
                Response::new(Status::Success, Vec::new())
            } else {
                Response::new(Status::CrcError, Vec::new())
            }
        }
        Err(_) => Response::new(Status::FlashError, Vec::new()),
    }
}
