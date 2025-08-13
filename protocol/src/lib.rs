#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crc::{Crc, CRC_16_IBM_SDLC};

/// CRC-16 calculator for packet integrity
pub const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

/// Magic numbers for packet synchronization
pub const PACKET_MAGIC: u16 = 0xABCD;
pub const RESPONSE_MAGIC: u16 = 0xDCBA;

/// Maximum data payload size per packet (optimized 64-byte packets)
pub const MAX_PAYLOAD_SIZE: usize = 64;

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
    /// CRC16 checksum
    pub crc: u16,
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
    /// CRC16 checksum
    pub crc: u16,
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
    pub fn calculate_crc(&self) -> u16 {
        let mut digest = CRC16.digest();
        digest.update(&self.magic.to_le_bytes());
        digest.update(&[self.command as u8]);
        digest.update(&self.length.to_le_bytes());
        digest.update(&self.address.to_le_bytes());
        digest.update(&self.sequence.to_le_bytes());
        digest.update(&self.data);
        digest.finalize()
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
        if bytes.len() < 15 {
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
            _ => return Err("Invalid command"),
        };

        let length = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
        let address = u32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]);
        let sequence = u16::from_le_bytes([bytes[11], bytes[12]]);

        if bytes.len() < 15 + length as usize {
            return Err("Incomplete packet");
        }

        let data = bytes[13..13 + length as usize].to_vec();
        let crc = u16::from_le_bytes([
            bytes[13 + length as usize],
            bytes[14 + length as usize],
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
    pub fn calculate_crc(&self) -> u16 {
        let mut digest = CRC16.digest();
        digest.update(&self.magic.to_le_bytes());
        digest.update(&[self.status as u8]);
        digest.update(&self.length.to_le_bytes());
        digest.update(&self.data);
        digest.finalize()
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
        if bytes.len() < 9 {
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

        if bytes.len() < 9 + length as usize {
            return Err("Incomplete response");
        }

        let data = bytes[7..7 + length as usize].to_vec();
        let crc = u16::from_le_bytes([
            bytes[7 + length as usize],
            bytes[8 + length as usize],
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
