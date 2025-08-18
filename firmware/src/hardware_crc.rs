use embassy_stm32::crc::Crc;
use flash_protocol::{Packet, Response};

/// Hardware CRC calculator for STM32G4
pub struct HardwareCrc {
    crc: Crc<'static>,
}

impl HardwareCrc {
    pub fn new(crc: Crc<'static>) -> Self {
        Self { crc }
    }

    /// Calculate CRC-32 for packet
    pub fn calculate_packet_crc(&mut self, packet: &Packet) -> u32 {
        self.crc.reset();

        // Create a buffer with all packet data in the same order as software CRC
        let mut buffer = heapless::Vec::<u8, 1024>::new();

        // Add fields in little-endian byte order (same as software)
        buffer.extend_from_slice(&packet.magic.to_le_bytes()).ok();
        buffer.push(packet.command as u8).ok();
        buffer.extend_from_slice(&packet.length.to_le_bytes()).ok();
        buffer.extend_from_slice(&packet.address.to_le_bytes()).ok();
        buffer
            .extend_from_slice(&packet.sequence.to_le_bytes())
            .ok();
        buffer.extend_from_slice(&packet.data).ok();

        // Feed all bytes to CRC
        self.feed_bytes(&buffer);

        self.crc.read()
    }

    /// Calculate CRC-32 for response
    pub fn calculate_response_crc(&mut self, response: &Response) -> u32 {
        self.crc.reset();

        // Create a buffer with all response data in the same order as software CRC
        let mut buffer = heapless::Vec::<u8, 1024>::new();

        // Add fields in little-endian byte order (same as software)
        buffer.extend_from_slice(&response.magic.to_le_bytes()).ok();
        buffer.push(response.status as u8).ok();
        buffer
            .extend_from_slice(&response.length.to_le_bytes())
            .ok();
        buffer.extend_from_slice(&response.data).ok();

        // Feed all bytes to CRC
        self.feed_bytes(&buffer);

        self.crc.read()
    }

    /// Feed bytes to CRC (handles non-word-aligned data)
    fn feed_bytes(&mut self, data: &[u8]) {
        // For now, use a simpler approach - feed bytes one by one
        // This is slower but more compatible
        for &byte in data {
            self.crc.feed_words(&[byte as u32]);
        }
    }
}

/// Global hardware CRC instance
static mut HARDWARE_CRC: Option<HardwareCrc> = None;

/// Initialize global hardware CRC
pub fn init_hardware_crc(crc: Crc<'static>) {
    unsafe {
        HARDWARE_CRC = Some(HardwareCrc::new(crc));
    }
}

/// Calculate CRC for packet using hardware
pub fn calculate_packet_crc(packet: &Packet) -> u32 {
    unsafe {
        if let Some(ref mut crc) = HARDWARE_CRC {
            crc.calculate_packet_crc(packet)
        } else {
            // Fallback if hardware CRC not initialized
            defmt::warn!("Hardware CRC not initialized, using fallback");
            0xDEADBEEF
        }
    }
}

/// Calculate CRC for response using hardware
pub fn calculate_response_crc(response: &Response) -> u32 {
    unsafe {
        if let Some(ref mut crc) = HARDWARE_CRC {
            crc.calculate_response_crc(response)
        } else {
            // Fallback if hardware CRC not initialized
            defmt::warn!("Hardware CRC not initialized, using fallback");
            0xBEEFDEAD
        }
    }
}

/// External function for protocol library (packet CRC)
#[no_mangle]
pub extern "Rust" fn calculate_packet_crc_external(packet: &Packet) -> u32 {
    calculate_packet_crc(packet)
}

/// External function for protocol library (response CRC)
#[no_mangle]
pub extern "Rust" fn calculate_response_crc_external(response: &Response) -> u32 {
    calculate_response_crc(response)
}
