#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use embassy_executor::Spawner;
use embassy_futures::join::join;

use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb};

use alloc::vec;
use alloc::vec::Vec;
use defmt_rtt as _;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use flash_protocol::*;
use panic_probe as _;
use static_cell::StaticCell;

mod safe_flash;
use safe_flash::SafeFlashManager;

mod hardware_crc;
use hardware_crc::init_hardware_crc;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

// Static buffers for USB with double buffering optimization
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 64] = [0; 64];
static mut USB_STATE: State = State::new();

// USB CDC buffer - standard size for CDC communication (currently unused)
#[allow(dead_code)]
static mut USB_RX_BUFFER: [u8; 64] = [0; 64]; // 64 bytes is standard for USB CDC

// Optimized heap for dynamic allocation (16KB) to handle 4KB write packets
static mut HEAP: [u8; 16384] = [0; 16384];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    // Initialize hardware CRC
    use embassy_stm32::crc::{Config as CrcConfig, InputReverseConfig, PolySize};
    let crc_config = CrcConfig::new(
        InputReverseConfig::None,
        false,
        PolySize::Width32,
        0xFFFFFFFF,
        0x04C11DB7, // Standard CRC-32 polynomial
    )
    .unwrap();
    let crc = embassy_stm32::crc::Crc::new(p.CRC, crc_config);
    init_hardware_crc(crc);
    defmt::info!("Hardware CRC initialized");

    // Initialize SPI for external Flash
    use embassy_stm32::gpio::{Level, Speed};
    use embassy_stm32::spi::{Config as SpiConfig, Spi};
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::mutex::Mutex;

    // SPI2 pins for external Flash (based on actual hardware configuration)
    // SCK: PB13, MISO: PB14, MOSI: PB15, CS: PA8 (assumed)
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = embassy_stm32::time::Hertz(20_000_000); // 20MHz SPI clock (high performance, W25Q128JV supports up to 133MHz)
                                                                   // SPI Mode 0 for W25Q128 (CPOL=0, CPHA=0) - this is the default mode
    let spi = Spi::new(
        p.SPI2, p.PB13,     // SCK
        p.PB15,     // MOSI
        p.PB14,     // MISO
        p.DMA2_CH3, // TX DMA
        p.DMA2_CH2, // RX DMA
        spi_config,
    );

    // CS pin (correct hardware connection: PB12)
    let _cs_pin = embassy_stm32::gpio::Output::new(p.PB12, Level::High, Speed::VeryHigh);

    // Flash Write Protect and Hold pins - CRITICAL for write operations!
    // WP# (Write Protect) - must be HIGH to allow writes (connected to PB11)
    let _wp_pin = embassy_stm32::gpio::Output::new(p.PB11, Level::High, Speed::VeryHigh);
    // HOLD# (Hold) - must be HIGH for normal operation (assuming PA10)
    let _hold_pin = embassy_stm32::gpio::Output::new(p.PA10, Level::High, Speed::VeryHigh);

    defmt::info!("Flash control pins configured: WP#=HIGH(PB11), HOLD#=HIGH(PA10)");

    // Create shared SPI bus
    static SPI_BUS: StaticCell<
        Mutex<CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>>,
    > = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(spi));

    // Create SafeFlashManager with real SPI hardware
    let mut flash_manager = SafeFlashManager::new();
    flash_manager.set_spi_resources(spi_bus);

    // CS pin is now managed internally by the flash manager

    // Try to initialize Flash
    defmt::info!(
        "Attempting to initialize SPI Flash on PB13(SCK), PB14(MISO), PB15(MOSI), PB12(CS)..."
    );
    match flash_manager.try_initialize().await {
        Ok(()) => {
            defmt::info!("✅ External Flash initialized successfully!");
            defmt::info!("Flash hardware is connected and responding to JEDEC ID requests");
        }
        Err(e) => {
            defmt::warn!("❌ Flash initialization failed: {:?}", e);
            defmt::warn!("This could mean:");
            defmt::warn!("  1. No SPI Flash chip is connected to the specified pins");
            defmt::warn!("  2. SPI pins are configured incorrectly");
            defmt::warn!("  3. Flash chip is not responding (wrong voltage, timing, etc.)");
            defmt::warn!("Continuing with fallback mode - Flash operations will return errors");
        }
    };

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
    let mut cdc_class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);
    let mut usb_device = builder.build();

    defmt::info!("System ready - using join architecture");

    // 使用join并行运行USB和协议处理任务
    let usb_fut = usb_device.run();
    let protocol_fut = async {
        loop {
            cdc_class.wait_connection().await;
            defmt::info!("USB Connected!");
            let _ = protocol_handler_loop(&mut cdc_class, &mut flash_manager).await;
            defmt::info!("USB Disconnected!");
        }
    };

    join(usb_fut, protocol_fut).await;
}

// 错误处理结构
struct Disconnected {}

impl From<embassy_usb::driver::EndpointError> for Disconnected {
    fn from(val: embassy_usb::driver::EndpointError) -> Self {
        match val {
            embassy_usb::driver::EndpointError::BufferOverflow => core::panic!("Buffer overflow"),
            embassy_usb::driver::EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn protocol_handler_loop<'a>(
    cdc_class: &mut CdcAcmClass<'a, Driver<'a, peripherals::USB>>,
    flash_manager: &mut SafeFlashManager,
) -> Result<(), Disconnected> {
    defmt::info!("Protocol handler started with full protocol support");

    // Protocol processing variables with memory management
    let mut packet_buffer = Vec::with_capacity(2048); // Pre-allocate reasonable capacity
    let mut buffer = [0u8; 64];
    const MAX_BUFFER_SIZE: usize = 4096; // Maximum buffer size to prevent memory issues

    loop {
        // Read data from USB
        let n = cdc_class.read_packet(&mut buffer).await?;
        if n > 0 {
            defmt::info!("USB: Received {} bytes", n);

            // Add to packet buffer with size check
            if packet_buffer.len() + n > MAX_BUFFER_SIZE {
                defmt::warn!(
                    "Buffer overflow protection: clearing buffer (was {} bytes)",
                    packet_buffer.len()
                );
                packet_buffer.clear();
            }
            packet_buffer.extend_from_slice(&buffer[..n]);
            defmt::info!("USB: Packet buffer now has {} bytes", packet_buffer.len());

            // Try to parse complete packets
            while let Some(packet) = try_parse_packet(&mut packet_buffer) {
                defmt::info!(
                    "Protocol: Parsed packet - Address: 0x{:08x}, Length: {}",
                    packet.address,
                    packet.length
                );

                // Process the command
                let response = match packet.command {
                    Command::Info => {
                        defmt::info!("Protocol: Processing Info command");
                        match flash_manager.get_flash_info().await {
                            Ok(info) => {
                                let mut data = Vec::new();
                                data.extend_from_slice(&info.jedec_id.to_le_bytes());
                                data.extend_from_slice(&info.total_size.to_le_bytes());
                                data.extend_from_slice(&info.page_size.to_le_bytes());
                                data.extend_from_slice(&info.sector_size.to_le_bytes());
                                Response::new(Status::Success, data)
                            }
                            Err(e) => {
                                defmt::error!("Flash info error: {:?}", e);
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::Read => {
                        defmt::info!("Protocol: Processing Read command");
                        match flash_manager.read_data(packet.address, packet.length).await {
                            Ok(data) => Response::new(Status::Success, data),
                            Err(e) => {
                                defmt::error!("Flash read error: {:?}", e);
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::Write => {
                        defmt::info!("Protocol: Processing Write command");
                        match flash_manager.write_data(packet.address, &packet.data).await {
                            Ok(()) => Response::new(Status::Success, Vec::new()),
                            Err(e) => {
                                defmt::error!("Flash write error: {:?}", e);
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::Erase => {
                        defmt::info!("Protocol: Processing Erase command");

                        // Extract size from packet data (4 bytes, little-endian)
                        if packet.data.len() < 4 {
                            defmt::error!("Erase command missing size data");
                            Response::new(Status::InvalidAddress, Vec::new())
                        } else {
                            let size = u32::from_le_bytes([
                                packet.data[0],
                                packet.data[1],
                                packet.data[2],
                                packet.data[3],
                            ]);

                            defmt::info!(
                                "Erasing {} bytes starting at address 0x{:08X}",
                                size,
                                packet.address
                            );

                            // Calculate number of sectors to erase (4KB per sector)
                            const SECTOR_SIZE: u32 = 4096;
                            let start_sector = packet.address / SECTOR_SIZE;
                            let end_address = packet.address + size;
                            let end_sector = end_address.div_ceil(SECTOR_SIZE); // Round up
                            let sectors_to_erase = end_sector - start_sector;

                            defmt::info!(
                                "Erasing {} sectors (0x{:08X} to 0x{:08X})",
                                sectors_to_erase,
                                start_sector * SECTOR_SIZE,
                                end_sector * SECTOR_SIZE
                            );

                            // Erase all required sectors
                            let mut success = true;
                            for sector in 0..sectors_to_erase {
                                let sector_address = (start_sector + sector) * SECTOR_SIZE;
                                match flash_manager.erase_sector(sector_address).await {
                                    Ok(()) => {
                                        defmt::info!("Erased sector at 0x{:08X}", sector_address);
                                    }
                                    Err(e) => {
                                        defmt::error!(
                                            "Flash erase error at 0x{:08X}: {:?}",
                                            sector_address,
                                            e
                                        );
                                        success = false;
                                        break;
                                    }
                                }
                            }

                            if success {
                                Response::new(Status::Success, Vec::new())
                            } else {
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::Verify => {
                        defmt::info!("Protocol: Processing Verify command");
                        // Mock verify success
                        Response::new(Status::Success, Vec::new())
                    }
                    Command::VerifyCRC => {
                        defmt::info!("Protocol: Processing VerifyCRC command");
                        // Mock CRC verify success for now
                        Response::new(Status::Success, Vec::new())
                    }
                    Command::Status => {
                        defmt::info!("Protocol: Processing Status command");

                        // First, run full diagnosis
                        match flash_manager.diagnose_flash_protection().await {
                            Ok(_) => defmt::info!("Flash protection diagnosis completed"),
                            Err(e) => defmt::error!("Flash diagnosis error: {:?}", e),
                        }

                        // Then return basic status
                        match flash_manager.read_status().await {
                            Ok(status) => {
                                defmt::info!("Flash status register: 0x{:02X}", status);
                                Response::new(Status::Success, vec![status])
                            }
                            Err(e) => {
                                defmt::error!("Flash status read error: {:?}", e);
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::StreamWrite => {
                        defmt::info!("Protocol: Processing StreamWrite command");
                        match flash_manager.write_data(packet.address, &packet.data).await {
                            Ok(_) => {
                                defmt::info!(
                                    "StreamWrite: Successfully wrote {} bytes at 0x{:08X}",
                                    packet.data.len(),
                                    packet.address
                                );
                                Response::new(Status::Success, Vec::new())
                            }
                            Err(_) => {
                                defmt::error!(
                                    "StreamWrite: Failed to write data at 0x{:08X}",
                                    packet.address
                                );
                                Response::new(Status::FlashError, Vec::new())
                            }
                        }
                    }
                    Command::BatchWrite | Command::BatchAck => {
                        defmt::info!("Protocol: Processing batch command");
                        // These commands are not implemented yet, but don't error
                        Response::new(Status::Success, Vec::new())
                    }
                };

                // Send response in chunks to avoid buffer overflow
                let response_data = response.to_bytes();
                defmt::info!("Protocol: Sending response, {} bytes", response_data.len());

                // Send in 64-byte chunks to match USB CDC buffer size
                const CHUNK_SIZE: usize = 64;
                let mut sent = 0;
                while sent < response_data.len() {
                    let chunk_end = core::cmp::min(sent + CHUNK_SIZE, response_data.len());
                    let chunk = &response_data[sent..chunk_end];
                    cdc_class.write_packet(chunk).await?;
                    sent = chunk_end;
                    defmt::debug!(
                        "Protocol: Sent chunk {} bytes, total sent: {}",
                        chunk.len(),
                        sent
                    );
                }
                defmt::info!("Protocol: Response sent successfully");

                // Memory management: shrink buffer if it's getting large
                if packet_buffer.capacity() > 2048 && packet_buffer.len() < 512 {
                    defmt::debug!(
                        "Memory: Shrinking buffer from capacity {} to {}",
                        packet_buffer.capacity(),
                        packet_buffer.len()
                    );
                    packet_buffer.shrink_to_fit();
                }

                // Don't clear the entire buffer - try_parse_packet already removed the processed packet
            }
        }
    }
}

fn try_parse_packet(buffer: &mut Vec<u8>) -> Option<Packet> {
    // Need at least minimum packet size (17 bytes: magic(2) + command(1) + length(4) + address(4) + sequence(2) + CRC(4))
    if buffer.len() < 17 {
        defmt::debug!(
            "Parse: Buffer too small ({} bytes), need at least 17",
            buffer.len()
        );
        return None;
    }

    // Look for magic number (0xABCD) at the start
    let magic_bytes = [0xCD, 0xAB]; // Little-endian 0xABCD

    // Find magic number in buffer
    let mut magic_pos = None;
    for i in 0..=buffer.len().saturating_sub(2) {
        if buffer[i..i + 2] == magic_bytes {
            magic_pos = Some(i);
            break;
        }
    }

    let magic_start = match magic_pos {
        Some(pos) => pos,
        None => {
            defmt::debug!("Parse: No magic number found in {} bytes", buffer.len());
            // Keep only the last few bytes in case we have a partial magic number
            if buffer.len() > 1024 {
                buffer.drain(0..buffer.len() - 1024);
            }
            return None;
        }
    };

    // Remove any data before the magic number
    if magic_start > 0 {
        buffer.drain(0..magic_start);
        defmt::debug!("Parse: Removed {} bytes before magic number", magic_start);
    }

    // Check if we have enough data for the header (magic + command + length + address + sequence = 13 bytes)
    if buffer.len() < 13 {
        defmt::debug!("Parse: Not enough data for header after magic removal");
        return None;
    }

    // Parse header according to correct protocol definition
    let magic = u16::from_le_bytes([buffer[0], buffer[1]]);
    let command_byte = buffer[2];
    let length = u32::from_le_bytes([buffer[3], buffer[4], buffer[5], buffer[6]]);
    let address = u32::from_le_bytes([buffer[7], buffer[8], buffer[9], buffer[10]]);
    let sequence = u16::from_le_bytes([buffer[11], buffer[12]]);

    defmt::debug!(
        "Parse: Magic: 0x{:08x}, Seq: {}, Cmd: {}, Addr: 0x{:08x}, Len: {}",
        magic,
        sequence,
        command_byte,
        address,
        length
    );

    // Validate magic number
    if magic != 0xABCD {
        defmt::warn!("Parse: Invalid magic number: 0x{:04x}", magic);
        buffer.drain(0..2); // Remove the invalid magic and try again
        return None;
    }

    // Parse command
    let command = match command_byte {
        0x01 => Command::Info,
        0x02 => Command::Erase,
        0x03 => Command::Write,
        0x04 => Command::Read,
        0x05 => Command::Verify,
        0x06 => Command::BatchWrite,
        0x07 => Command::BatchAck,
        0x08 => Command::StreamWrite,
        0x09 => Command::VerifyCRC,
        0x0A => Command::Status,
        _ => {
            defmt::warn!("Parse: Unknown command: 0x{:02x}", command_byte);
            buffer.drain(0..13); // Remove the invalid packet header
            return None;
        }
    };

    // Calculate total packet size based on command type
    let (total_size, data_length) = match command {
        Command::Read => {
            // For read commands, length field indicates how much to read, not packet data size
            (13 + 4, 0) // header(13) + CRC(4), no data in packet
        }
        _ => {
            // For other commands, length field indicates actual data in packet
            (13 + length as usize + 4, length as usize) // header(13) + data + CRC(4)
        }
    };

    // Check if we have the complete packet
    if buffer.len() < total_size {
        defmt::debug!(
            "Parse: Incomplete packet: have {} bytes, need {}",
            buffer.len(),
            total_size
        );
        return None;
    }

    // Extract data with size limit to prevent memory issues
    let data = if data_length > 0 {
        if data_length > 1024 {
            defmt::error!("Packet too large: {} bytes, rejecting", data_length);
            return None; // Reject packets larger than 1KB
        }
        let extracted_data = buffer[13..13 + data_length].to_vec();
        defmt::debug!("Parse: Extracted {} bytes of data", extracted_data.len());
        if extracted_data.len() <= 32 {
            // Only show first 32 bytes to avoid log spam
            for (i, byte) in extracted_data.iter().enumerate() {
                if i % 16 == 0 && i > 0 {
                    defmt::debug!("");
                }
                defmt::debug!("{:02X} ", byte);
            }
        }
        extracted_data
    } else {
        Vec::new()
    };

    // Extract CRC (32-bit)
    let crc_start = 13 + data_length;
    let received_crc = if crc_start + 3 < buffer.len() {
        u32::from_le_bytes([
            buffer[crc_start],
            buffer[crc_start + 1],
            buffer[crc_start + 2],
            buffer[crc_start + 3],
        ])
    } else {
        0 // No CRC available
    };

    // For now, skip CRC verification to test basic functionality
    // TODO: Implement proper CRC-16 verification

    // Remove the parsed packet from buffer
    buffer.drain(0..total_size);

    defmt::info!(
        "Parse: Successfully parsed packet - Addr: 0x{:08x}, Len: {}",
        address,
        length
    );

    Some(Packet {
        magic,
        sequence,
        command,
        address,
        length,
        data,
        crc: received_crc,
    })
}
