#![no_std]
#![no_main]

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

mod flash_driver;
mod protocol_handler;

use flash_driver::FlashDriver;
use protocol_handler::ProtocolHandler;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

type SharedSpi = Mutex<CriticalSectionRawMutex, Spi<'static, peripherals::SPI2>>;
type FlashSpiDevice = SpiDevice<'static, CriticalSectionRawMutex, Spi<'static, peripherals::SPI2>, Output<'static>>;

static SPI_BUS: Mutex<CriticalSectionRawMutex, Option<Spi<'static, peripherals::SPI2>>> = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    
    let mut handler = ProtocolHandler::new();
    let mut buffer = [0u8; 1024];
    
    loop {
        // Try to read data
        match cdc_class.read_packet(&mut buffer).await {
            Ok(n) if n > 0 => {
                info!("Received {} bytes", n);
                
                // Process the packet
                match handler.process_packet(&buffer[..n], &mut flash_driver).await {
                    Ok(response_data) => {
                        // Send response
                        if let Err(e) = cdc_class.write_packet(&response_data).await {
                            error!("Failed to send response: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to process packet: {:?}", e);
                        // Send error response
                        let error_response = Response::new(Status::Unknown, alloc::vec::Vec::new());
                        let response_data = error_response.to_bytes();
                        let _ = cdc_class.write_packet(&response_data).await;
                    }
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
