#![no_std]
#![no_main]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use defmt::*;
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

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

// Simple heap for dynamic allocation (8KB)
static mut HEAP: [u8; 8192] = [0; 8192];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP.len());
    }

    info!("STM32G4 Flash Programmer starting...");

    // Initialize hardware with default config
    let p = embassy_stm32::init(Config::default());
    info!("Hardware initialized");

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
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();
    let mut builder = Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // Create CDC-ACM class
    let cdc_class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let usb_device = builder.build();

    info!("USB initialized");

    // Spawn USB task
    spawner.spawn(usb_task(usb_device)).unwrap();

    // Spawn protocol handler task
    spawner.spawn(protocol_task(cdc_class)).unwrap();

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
                    
                    // Process the command (mock implementation for now)
                    let response = match packet.command {
                        Command::Info => {
                            let mut data = Vec::new();
                            data.extend_from_slice(&0xEF4018u32.to_le_bytes()); // Mock JEDEC ID
                            data.extend_from_slice(&(16 * 1024 * 1024u32).to_le_bytes()); // 16MB
                            data.extend_from_slice(&256u32.to_le_bytes()); // Page size
                            data.extend_from_slice(&4096u32.to_le_bytes()); // Sector size
                            Response::new(Status::Success, data)
                        }
                        Command::Erase => {
                            info!("Mock erase at address 0x{:08X}", packet.address);
                            Response::new(Status::Success, Vec::new())
                        }
                        Command::Write => {
                            info!("Mock write {} bytes at address 0x{:08X}", packet.data.len(), packet.address);
                            Response::new(Status::Success, Vec::new())
                        }
                        Command::Read => {
                            let size = if packet.data.len() >= 4 {
                                u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
                            } else {
                                continue;
                            };
                            info!("Mock read {} bytes from address 0x{:08X}", size, packet.address);
                            let mock_data = vec![0xAAu8; size as usize];
                            Response::new(Status::Success, mock_data)
                        }
                        Command::Verify => {
                            info!("Mock verify {} bytes at address 0x{:08X}", packet.data.len(), packet.address);
                            Response::new(Status::Success, Vec::new())
                        }
                    };
                    
                    // Send response
                    let response_data = response.to_bytes();
                    if let Err(_e) = cdc_class.write_packet(&response_data).await {
                        error!("Failed to send response");
                    }
                    
                    // Clear packet buffer
                    packet_buffer.clear();
                }
            }
            Ok(_) => {
                // No data received, continue
            }
            Err(_e) => {
                warn!("USB read error");
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        
        Timer::after(Duration::from_millis(10)).await;
    }
}
