#![no_std]
#![no_main]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use flash_protocol::*;
use panic_halt as _;
use alloc::vec::Vec;
use alloc::vec;

mod safe_flash;
use safe_flash::SafeFlashManager;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

// Static buffers for USB
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 64] = [0; 64];
static mut USB_STATE: State = State::new();

// Optimized heap for dynamic allocation (16KB) to handle 4KB write packets
static mut HEAP: [u8; 16384] = [0; 16384];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP.len());
    }

    // Initialize hardware with default config
    let p = embassy_stm32::init(Config::default());

    // Create SafeFlashManager (no SPI initialization yet)
    let mut flash_manager = SafeFlashManager::new();

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

    // Spawn protocol handler task with Flash manager
    spawner.spawn(protocol_task(cdc_class, flash_manager)).unwrap();

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
    mut flash_manager: SafeFlashManager,
) {
    // Wait a bit for USB to be ready
    Timer::after(Duration::from_secs(2)).await;
    
    let mut buffer = [0u8; 1024];
    let mut packet_buffer = Vec::new();
    
    loop {
        // Try to read data
        match cdc_class.read_packet(&mut buffer).await {
            Ok(n) if n > 0 => {
                // Add to packet buffer
                packet_buffer.extend_from_slice(&buffer[..n]);
                
                // Try to parse a complete packet
                while let Some(packet) = try_parse_packet(&mut packet_buffer) {
                    // Process the command (mock implementation for now)
                    let response = match packet.command {
                        Command::Info => {
                            // Try to get real Flash info, fallback to mock if failed
                            match flash_manager.get_flash_info().await {
                                Ok(flash_info) => {
                                    let mut data = Vec::new();
                                    data.extend_from_slice(&flash_info.jedec_id.to_le_bytes());
                                    data.extend_from_slice(&flash_info.total_size.to_le_bytes());
                                    data.extend_from_slice(&flash_info.page_size.to_le_bytes());
                                    data.extend_from_slice(&flash_info.sector_size.to_le_bytes());
                                    Response::new(Status::Success, data)
                                }
                                Err(_) => {
                                    // Fallback to mock data if Flash read fails
                                    let mut data = Vec::new();
                                    data.extend_from_slice(&0xEF4018u32.to_le_bytes()); // Mock JEDEC ID
                                    data.extend_from_slice(&(16 * 1024 * 1024u32).to_le_bytes()); // 16MB
                                    data.extend_from_slice(&256u32.to_le_bytes()); // Page size
                                    data.extend_from_slice(&4096u32.to_le_bytes()); // Sector size
                                    Response::new(Status::Success, data)
                                }
                            }
                        }
                        Command::Erase => {
                            // Mock erase operation
                            Response::new(Status::Success, Vec::new())
                        }
                        Command::Write => {
                            // Check data size to prevent memory issues
                            if packet.data.len() > 8192 {
                                // Data too large, return error
                                Response::new(Status::BufferOverflow, Vec::new())
                            } else {
                                // TODO: Implement real Flash write operation
                                // For now, simulate successful write
                                // In production: flash_driver.write_data(packet.address, &packet.data).await
                                Response::new(Status::Success, Vec::new())
                            }
                        }
                        Command::Read => {
                            // Use packet.length as the size to read
                            let size = packet.length as usize;
                            // Mock read operation - return pattern data
                            let mock_data = vec![0x42u8; size];
                            Response::new(Status::Success, mock_data)
                        }
                        Command::Verify => {
                            // Mock verify operation
                            Response::new(Status::Success, Vec::new())
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
            }
            Ok(_) => {
                // No data received, continue
            }
            Err(_e) => {
                // USB read error
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        
        Timer::after(Duration::from_millis(10)).await;
    }
}

fn try_parse_packet(buffer: &mut Vec<u8>) -> Option<Packet> {
    // Need at least minimum packet size (15 bytes with sequence number)
    if buffer.len() < 15 {
        return None;
    }

    // Look for magic number
    for i in 0..buffer.len().saturating_sub(1) {
        let magic = u16::from_le_bytes([buffer[i], buffer[i + 1]]);
        if magic == PACKET_MAGIC {
            // Found potential packet start
            if let Ok(packet) = Packet::from_bytes(&buffer[i..]) {
                // Calculate packet size and remove from buffer
                let packet_size = packet.to_bytes().len();
                buffer.drain(0..i + packet_size);
                return Some(packet);
            }
        }
    }

    // Keep only the last few bytes in case we have a partial magic number
    if buffer.len() > 1024 {
        buffer.drain(0..buffer.len() - 1024);
    }

    None
}
