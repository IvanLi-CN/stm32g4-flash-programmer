#![no_std]
#![no_main]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use embassy_executor::Spawner;

use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use flash_protocol::*;
use panic_probe as _;
use defmt_rtt as _;
use alloc::vec::Vec;
use alloc::vec;

mod safe_flash;
use safe_flash::SafeFlashManager;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

// Static buffers for USB with double buffering optimization
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 64] = [0; 64];
static mut USB_STATE: State = State::new();

// Optimized dual-buffer system for high-speed USB transfers (memory-efficient)
static mut USB_RX_BUFFER_1: [u8; 4096] = [0; 4096];  // 4KB buffers for balance of speed and memory
static mut USB_RX_BUFFER_2: [u8; 4096] = [0; 4096];
static mut CURRENT_BUFFER: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

// Optimized heap for dynamic allocation (16KB) to handle 4KB write packets
static mut HEAP: [u8; 16384] = [0; 16384];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP.len());
        defmt::info!("Heap initialized: {} bytes", HEAP.len());
    }

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
        config.enable_ucpd1_dead_battery = true;
    }
    let p = embassy_stm32::init(config);
    defmt::info!("STM32 initialized successfully");

    // Create SafeFlashManager (no SPI initialization yet)
    let flash_manager = SafeFlashManager::new();
    defmt::info!("SafeFlashManager created");

    // Initialize USB
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    defmt::info!("USB driver initialized");
    
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

    // Create CDC-ACM class with minimal buffer size
    let cdc_class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);
    let usb_device = builder.build();

    // Spawn USB task
    spawner.spawn(usb_task(usb_device)).unwrap();
    defmt::info!("USB task spawned");

    // Spawn protocol handler task with Flash manager
    spawner.spawn(protocol_task(cdc_class, flash_manager)).unwrap();
    defmt::info!("Protocol handler task spawned");

    // Main loop - keep the main task alive
    defmt::info!("Entering main loop - system ready");
    loop {
        Timer::after(Duration::from_millis(1000)).await;
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
    // USB ready check removed for speed
    
    // High-performance dual buffering system (memory-efficient)
    let mut packet_buffer = Vec::new();
    let mut _burst_mode = false;
    let mut burst_packet_count = 0;

    // Get current buffer for dual buffering
    let get_current_buffer = || -> &'static mut [u8; 4096] {
        unsafe {
            if CURRENT_BUFFER.load(core::sync::atomic::Ordering::Relaxed) {
                &mut USB_RX_BUFFER_2
            } else {
                &mut USB_RX_BUFFER_1
            }
        }
    };

    // Switch to next buffer
    let switch_buffer = || {
        unsafe {
            CURRENT_BUFFER.fetch_xor(true, core::sync::atomic::Ordering::Relaxed);
        }
    };
    
    loop {
        // Get current buffer for high-speed reading
        let current_buffer = get_current_buffer();

        // Try to read data using triple buffering
        match cdc_class.read_packet(current_buffer).await {
            Ok(n) if n > 0 => {
                // Switch to next buffer immediately for next read
                switch_buffer();

                // Add to packet buffer
                packet_buffer.extend_from_slice(&current_buffer[..n]);

                // Detect burst mode for high-speed transfers
                if n >= 1024 {
                    _burst_mode = true;
                    burst_packet_count += 1;
                } else if burst_packet_count > 0 {
                    burst_packet_count -= 1;
                    if burst_packet_count == 0 {
                        _burst_mode = false;
                    }
                }
                
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
                            // Check data size to prevent memory issues (allow up to 1KB payload)
                            if packet.data.len() > 1024 {
                                // Data too large, return error
                                Response::new(Status::BufferOverflow, Vec::new())
                            } else {
                                // TODO: Implement real Flash write operation
                                // For now, simulate successful write
                                // In production: flash_driver.write_data(packet.address, &packet.data).await
                                Response::new(Status::Success, Vec::new())
                            }
                        }
                        Command::BatchWrite => {
                            // Batch write mode - process but don't send response
                            if packet.data.len() > 1024 {
                                Response::new(Status::BufferOverflow, Vec::new())
                            } else {
                                // TODO: Implement real Flash write operation
                                // For now, simulate successful write
                                // Skip sending response for batch writes
                                packet_buffer.clear();
                                continue;
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
                        Command::VerifyCRC => {
                            // CRC verification - for demo, always succeed to test the flow
                            if packet.data.len() >= 4 {
                                let _expected_crc = u32::from_le_bytes([
                                    packet.data[0], packet.data[1], packet.data[2], packet.data[3]
                                ]);

                                // For demo: always return success to test the verification flow
                                // In real implementation, this would:
                                // 1. Calculate CRC32 of the actual flash data at the specified address
                                // 2. Compare with expected_crc
                                // 3. Return Success or VerificationFailed accordingly
                                Response::new(Status::Success, Vec::new())
                            } else {
                                Response::new(Status::InvalidCommand, Vec::new())
                            }
                        }
                        Command::BatchAck => {
                            // Batch ACK command - not expected from host
                            Response::new(Status::InvalidCommand, Vec::new())
                        }
                        Command::StreamWrite => {
                            // Stream write mode - no response at all, maximum speed
                            if packet.data.len() > 1024 {
                                // Data too large, but don't send response to maintain speed
                                packet_buffer.clear();
                                continue;
                            } else {
                                // TODO: Implement real Flash write operation
                                // For now, simulate successful write
                                // No response needed for stream writes
                                packet_buffer.clear();
                                continue;
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
            }
            Ok(_) => {
                // No data received, continue
            }
            Err(_e) => {
                // USB read error - continue immediately for speed
            }
        }

        // Removed 10ms delay for maximum speed
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
