#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// Hardware CRC-32 will be used on STM32 side
// Software fallback for host tools
#[cfg(feature = "std")]
use crc::{Crc, CRC_32_ISO_HDLC};

#[cfg(feature = "std")]
/// CRC-32 calculator for packet integrity (software fallback)
pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// Magic numbers for packet synchronization
pub const PACKET_MAGIC: u16 = 0xABCD;
pub const RESPONSE_MAGIC: u16 = 0xDCBA;

/// Maximum data payload size per packet (optimized for speed and stability - 1KB packets)
pub const MAX_PAYLOAD_SIZE: usize = 1024;

/// Flash page size for W25Q128 (256 bytes)
pub const FLASH_PAGE_SIZE: usize = 256;

/// Flash sector size for W25Q128 (4KB)
pub const FLASH_SECTOR_SIZE: usize = 4096;

/// Total flash size for W25Q128 (16MB)
pub const FLASH_TOTAL_SIZE: usize = 16 * 1024 * 1024;

/// Command types for flash operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// Get flash information (size, page size, etc.)
    Info = 0x01,
    /// Erase flash sector(s)
    Erase = 0x02,
    /// Write data to flash
    Write = 0x03,
    /// Read data from flash
    Read = 0x04,
    /// Verify data integrity
    Verify = 0x05,
    /// Batch write mode - no immediate ACK required
    BatchWrite = 0x06,
    /// Batch ACK - acknowledge multiple packets
    BatchAck = 0x07,
    /// Stream write - no ACK at all, maximum speed
    StreamWrite = 0x08,
    /// Verify data integrity using CRC32
    VerifyCRC = 0x09,
    /// Read flash status register
    Status = 0x0A,
}

/// Status codes for responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    /// Operation completed successfully
    Success = 0x00,
    /// Invalid command
    InvalidCommand = 0x01,
    /// Invalid address or size
    InvalidAddress = 0x02,
    /// Flash operation failed
    FlashError = 0x03,
    /// CRC mismatch
    CrcError = 0x04,
    /// Buffer overflow
    BufferOverflow = 0x05,
    /// Operation timeout
    Timeout = 0x06,
    /// Data verification failed
    VerificationFailed = 0x07,
    /// Unknown error
    Unknown = 0xFF,
}

/// Command packet structure
#[derive(Debug, Clone)]
pub struct Packet {
    /// Magic number for synchronization
    pub magic: u16,
    /// Command type
    pub command: Command,
    /// Data length
    pub length: u32,
    /// Flash address (for read/write/erase operations)
    pub address: u32,
    /// Sequence number for packet ordering and acknowledgment
    pub sequence: u16,
    /// Data payload
    pub data: Vec<u8>,
    /// CRC32 checksum
    pub crc: u32,
}

/// Response packet structure
#[derive(Debug, Clone)]
pub struct Response {
    /// Magic number for synchronization
    pub magic: u16,
    /// Status code
    pub status: Status,
    /// Response data length
    pub length: u32,
    /// Response data
    pub data: Vec<u8>,
    /// CRC32 checksum
    pub crc: u32,
}

impl Packet {
    /// Create a new packet
    pub fn new(command: Command, address: u32, data: Vec<u8>) -> Self {
        Self::new_with_sequence(command, address, data, 0)
    }

    /// Create a new packet with sequence number
    pub fn new_with_sequence(command: Command, address: u32, data: Vec<u8>, sequence: u16) -> Self {
        let mut packet = Self {
            magic: PACKET_MAGIC,
            command,
            length: data.len() as u32,
            address,
            sequence,
            data,
            crc: 0,
        };
        packet.crc = packet.calculate_crc();
        packet
    }

    /// Calculate CRC for the packet
    #[cfg(feature = "std")]
    pub fn calculate_crc(&self) -> u32 {
        let mut digest = CRC32.digest();
        digest.update(&self.magic.to_le_bytes());
        digest.update(&[self.command as u8]);
        digest.update(&self.length.to_le_bytes());
        digest.update(&self.address.to_le_bytes());
        digest.update(&self.sequence.to_le_bytes());
        digest.update(&self.data);
        digest.finalize()
    }

    /// Calculate CRC for the packet (no-std version, temporary software fallback)
    #[cfg(not(feature = "std"))]
    pub fn calculate_crc(&self) -> u32 {
        // Temporary software CRC implementation for compatibility
        // TODO: Re-enable hardware CRC after debugging
        let mut crc = 0xFFFFFFFFu32;

        // Simple CRC-32 calculation (not optimized, but compatible)
        let data = [
            &self.magic.to_le_bytes()[..],
            &[self.command as u8],
            &self.length.to_le_bytes()[..],
            &self.address.to_le_bytes()[..],
            &self.sequence.to_le_bytes()[..],
            &self.data[..],
        ].concat();

        for &byte in &data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }

        !crc
    }

    /// Verify packet integrity
    pub fn verify_crc(&self) -> bool {
        self.crc == self.calculate_crc()
    }

    /// Serialize packet to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.push(self.command as u8);
        bytes.extend_from_slice(&self.length.to_le_bytes());
        bytes.extend_from_slice(&self.address.to_le_bytes());
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_le_bytes());
        bytes
    }

    /// Deserialize packet from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 17 {
            return Err("Packet too short");
        }

        let magic = u16::from_le_bytes([bytes[0], bytes[1]]);
        if magic != PACKET_MAGIC {
            return Err("Invalid magic number");
        }

        let command = match bytes[2] {
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
            _ => return Err("Invalid command"),
        };

        let length = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
        let address = u32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]);
        let sequence = u16::from_le_bytes([bytes[11], bytes[12]]);

        if bytes.len() < 17 + length as usize {
            return Err("Incomplete packet");
        }

        let data = bytes[13..13 + length as usize].to_vec();
        let crc = u32::from_le_bytes([
            bytes[13 + length as usize],
            bytes[14 + length as usize],
            bytes[15 + length as usize],
            bytes[16 + length as usize],
        ]);

        let packet = Self {
            magic,
            command,
            length,
            address,
            sequence,
            data,
            crc,
        };

        if !packet.verify_crc() {
            return Err("CRC mismatch");
        }

        Ok(packet)
    }
}

impl Response {
    /// Create a new response
    pub fn new(status: Status, data: Vec<u8>) -> Self {
        let mut response = Self {
            magic: RESPONSE_MAGIC,
            status,
            length: data.len() as u32,
            data,
            crc: 0,
        };
        response.crc = response.calculate_crc();
        response
    }

    /// Calculate CRC for the response
    #[cfg(feature = "std")]
    pub fn calculate_crc(&self) -> u32 {
        let mut digest = CRC32.digest();
        digest.update(&self.magic.to_le_bytes());
        digest.update(&[self.status as u8]);
        digest.update(&self.length.to_le_bytes());
        digest.update(&self.data);
        digest.finalize()
    }

    /// Calculate CRC for the response (no-std version, temporary software fallback)
    #[cfg(not(feature = "std"))]
    pub fn calculate_crc(&self) -> u32 {
        // Temporary software CRC implementation for compatibility
        // TODO: Re-enable hardware CRC after debugging
        let mut crc = 0xFFFFFFFFu32;

        // Simple CRC-32 calculation (not optimized, but compatible)
        let data = [
            &self.magic.to_le_bytes()[..],
            &[self.status as u8],
            &self.length.to_le_bytes()[..],
            &self.data[..],
        ].concat();

        for &byte in &data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }

        !crc
    }

    /// Verify response integrity
    pub fn verify_crc(&self) -> bool {
        self.crc == self.calculate_crc()
    }

    /// Serialize response to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.push(self.status as u8);
        bytes.extend_from_slice(&self.length.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_le_bytes());
        bytes
    }

    /// Deserialize response from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 11 {
            return Err("Response too short");
        }

        let magic = u16::from_le_bytes([bytes[0], bytes[1]]);
        if magic != RESPONSE_MAGIC {
            return Err("Invalid magic number");
        }

        let status = match bytes[2] {
            0x00 => Status::Success,
            0x01 => Status::InvalidCommand,
            0x02 => Status::InvalidAddress,
            0x03 => Status::FlashError,
            0x04 => Status::CrcError,
            0x05 => Status::BufferOverflow,
            0x06 => Status::Timeout,
            _ => Status::Unknown,
        };

        let length = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);

        if bytes.len() < 11 + length as usize {
            return Err("Incomplete response");
        }

        let data = bytes[7..7 + length as usize].to_vec();
        let crc = u32::from_le_bytes([
            bytes[7 + length as usize],
            bytes[8 + length as usize],
            bytes[9 + length as usize],
            bytes[10 + length as usize],
        ]);

        let response = Self {
            magic,
            status,
            length,
            data,
            crc,
        };

        if !response.verify_crc() {
            return Err("CRC mismatch");
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let packet = Packet::new(Command::Write, 0x1000, data.clone());
        
        let bytes = packet.to_bytes();
        let decoded = Packet::from_bytes(&bytes).unwrap();
        
        assert_eq!(packet.command, decoded.command);
        assert_eq!(packet.address, decoded.address);
        assert_eq!(packet.data, decoded.data);
        assert!(decoded.verify_crc());
    }

    #[test]
    fn test_response_serialization() {
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let response = Response::new(Status::Success, data.clone());
        
        let bytes = response.to_bytes();
        let decoded = Response::from_bytes(&bytes).unwrap();
        
        assert_eq!(response.status, decoded.status);
        assert_eq!(response.data, decoded.data);
        assert!(decoded.verify_crc());
    }
}
