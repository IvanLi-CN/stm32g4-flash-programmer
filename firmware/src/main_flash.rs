#![no_std]
#![no_main]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as EmbassySpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi as Stm32Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, mode, peripherals, usb, Config};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use embedded_storage_async::nor_flash::NorFlash;
use flash_protocol::*;
use panic_halt as _;
use static_cell::StaticCell;
use w25::{Q, W25};
use alloc::vec::Vec;
use alloc::vec;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
    SPI2 => embassy_stm32::spi::InterruptHandler<peripherals::SPI2>;
});

// Static buffers for USB
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 64] = [0; 64];
static mut USB_STATE: State = State::new();

// Simple heap for dynamic allocation (8KB)
static mut HEAP: [u8; 8192] = [0; 8192];

/// Dummy pin implementation for W25 driver
pub struct DummyPin;

#[derive(Debug)]
pub struct DummyError;

impl embedded_hal::digital::Error for DummyError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for DummyPin {
    type Error = DummyError;
}

impl embedded_hal::digital::OutputPin for DummyPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

type FlashType = W25<
    Q,
    EmbassySpiDevice<
        'static,
        CriticalSectionRawMutex,
        Stm32Spi<'static, mode::Async>,
        Output<'static>,
    >,
    DummyPin,
    DummyPin,
>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP.len());
    }

    // Initialize hardware with default config
    let p = embassy_stm32::init(Config::default());

    // Initialize SPI Flash
    let flash = initialize_flash_spi(p.SPI2, p.PB13, p.PB15, p.PB14, p.PB12, p.PB11, p.DMA1_CH4, p.DMA1_CH5).await;

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

    // Create embassy-usb DeviceBuilder using static buffers
    let mut builder = Builder::new(
        driver,
        usb_config,
        unsafe { &mut CONFIG_DESCRIPTOR },
        unsafe { &mut BOS_DESCRIPTOR },
        &mut [], // no msos descriptors
        unsafe { &mut CONTROL_BUF },
    );

    // Create CDC-ACM class
    let cdc_class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);
    let usb_device = builder.build();

    // Spawn USB task
    spawner.spawn(usb_task(usb_device)).unwrap();

    // Spawn protocol handler task with flash
    spawner.spawn(protocol_task(cdc_class, flash)).unwrap();

    // Main loop - blink LED to show we're alive
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(250)).await;
        led.set_low();
        Timer::after(Duration::from_millis(250)).await;
    }
}

/// Initialize SPI2 for W25Q128 Flash communication
async fn initialize_flash_spi(
    spi2: peripherals::SPI2,
    sck: peripherals::PB13,
    mosi: peripherals::PB15,
    miso: peripherals::PB14,
    cs: peripherals::PB12,
    wp: peripherals::PB11,
    dma_tx: peripherals::DMA1_CH4,
    dma_rx: peripherals::DMA1_CH5,
) -> FlashType {
    // Configure CS pin
    let cs_pin_output = Output::new(cs, Level::High, Speed::VeryHigh);

    // Configure WP (Write Protect) pin - CRITICAL for Flash writing!
    let _wp_pin_output = Output::new(wp, Level::High, Speed::VeryHigh);
    let wp_pin = DummyPin;
    let hold_pin = DummyPin;

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(8_000_000); // 8MHz for Flash communication

    let spi_bus = Stm32Spi::new(spi2, sck, mosi, miso, dma_tx, dma_rx, spi_config);

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

    match W25::new(spi_device, hold_pin, wp_pin, 16 * 1024 * 1024) {
        Ok(flash) => flash,
        Err(_e) => {
            core::panic!("Flash initialization failed");
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb_device: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    usb_device.run().await;
}

#[embassy_executor::task]
async fn protocol_task(
    mut cdc_class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
    mut flash: FlashType,
) {
    // Wait a bit for USB to be ready
    Timer::after(Duration::from_secs(3)).await;
    
    let mut buffer = [0u8; 1024];
    let mut packet_buffer = Vec::new();
    
    loop {
        // Try to read data
        match cdc_class.read_packet(&mut buffer).await {
            Ok(n) if n > 0 => {
                // Add to packet buffer
                packet_buffer.extend_from_slice(&buffer[..n]);
                
                // Try to parse a complete packet
                match Packet::from_bytes(&packet_buffer) {
                    Ok(packet) => {
                        // Process the command with real Flash operations
                        let response = match packet.command {
                            Command::Info => {
                                let mut data = Vec::new();
                                data.extend_from_slice(&0xEF4018u32.to_le_bytes()); // W25Q128 JEDEC ID
                                data.extend_from_slice(&(16 * 1024 * 1024u32).to_le_bytes()); // 16MB
                                data.extend_from_slice(&256u32.to_le_bytes()); // Page size
                                data.extend_from_slice(&4096u32.to_le_bytes()); // Sector size
                                Response::new(Status::Success, data)
                            }
                            Command::Erase => {
                                // Real erase operation
                                match flash.erase(packet.address, packet.address + 4096).await {
                                    Ok(_) => Response::new(Status::Success, Vec::new()),
                                    Err(_) => Response::new(Status::Error, Vec::new()),
                                }
                            }
                            Command::Write => {
                                // Real write operation
                                match flash.write(packet.address, &packet.data).await {
                                    Ok(_) => Response::new(Status::Success, Vec::new()),
                                    Err(_) => Response::new(Status::Error, Vec::new()),
                                }
                            }
                            Command::Read => {
                                let size = if packet.data.len() >= 4 {
                                    u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
                                } else {
                                    256
                                };
                                let mut read_data = vec![0u8; size as usize];
                                match flash.read(packet.address, &mut read_data).await {
                                    Ok(_) => Response::new(Status::Success, read_data),
                                    Err(_) => Response::new(Status::Error, Vec::new()),
                                }
                            }
                            Command::Verify => {
                                // Real verify operation
                                let mut read_data = vec![0u8; packet.data.len()];
                                match flash.read(packet.address, &mut read_data).await {
                                    Ok(_) => {
                                        if read_data == packet.data {
                                            Response::new(Status::Success, Vec::new())
                                        } else {
                                            Response::new(Status::Error, Vec::new())
                                        }
                                    }
                                    Err(_) => Response::new(Status::Error, Vec::new()),
                                }
                            }
                        };
                        
                        // Send response
                        let response_data = response.to_bytes();
                        if let Err(_e) = cdc_class.write_packet(&response_data).await {
                            // Error sending response
                        }
                        
                        // Clear packet buffer
                        packet_buffer.clear();
                    }
                    Err(_) => {
                        // Not a complete packet yet, or parse error
                        if packet_buffer.len() > 2048 {
                            packet_buffer.clear();
                        }
                    }
                }
            }
            Ok(_) => {
                // No data received, continue
            }
            Err(_e) => {
                // USB read error
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        
        Timer::after(Duration::from_millis(1)).await;
    }
}
