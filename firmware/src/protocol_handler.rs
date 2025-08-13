use defmt::*;
use embedded_hal_async::spi::SpiDevice;
use flash_protocol::*;
use heapless::Vec as HVec;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::flash_driver::{FlashDriver, FlashDriverError};

pub struct ProtocolHandler {
    buffer: HVec<u8, 8192>, // 8KB buffer for packet assembly
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            buffer: HVec::new(),
        }
    }

    pub async fn process_packet<SPI>(
        &mut self,
        data: &[u8],
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Result<Vec<u8>, &'static str>
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        // Add received data to buffer
        for &byte in data {
            if self.buffer.push(byte).is_err() {
                // Buffer overflow, reset and return error
                self.buffer.clear();
                let response = Response::new(Status::BufferOverflow, Vec::new());
                return Ok(response.to_bytes());
            }
        }

        // Try to parse a complete packet
        match self.try_parse_packet() {
            Ok(Some(packet)) => {
                info!("Processing command: {:?}", packet.command);
                let response = self.handle_command(packet, flash_driver).await;
                Ok(response.to_bytes())
            }
            Ok(None) => {
                // Incomplete packet, wait for more data
                Err("Incomplete packet")
            }
            Err(e) => {
                // Parse error, clear buffer and return error response
                warn!("Packet parse error: {}", e);
                self.buffer.clear();
                let response = Response::new(Status::CrcError, Vec::new());
                Ok(response.to_bytes())
            }
        }
    }

    fn try_parse_packet(&mut self) -> Result<Option<Packet>, &'static str> {
        if self.buffer.len() < 13 {
            // Minimum packet size not reached
            return Ok(None);
        }

        // Look for magic number
        let mut magic_pos = None;
        for i in 0..=self.buffer.len().saturating_sub(2) {
            let magic = u16::from_le_bytes([self.buffer[i], self.buffer[i + 1]]);
            if magic == PACKET_MAGIC {
                magic_pos = Some(i);
                break;
            }
        }

        let magic_pos = match magic_pos {
            Some(pos) => pos,
            None => {
                // No magic found, keep only the last byte in case it's part of magic
                if self.buffer.len() > 1 {
                    let last_byte = self.buffer[self.buffer.len() - 1];
                    self.buffer.clear();
                    let _ = self.buffer.push(last_byte);
                }
                return Ok(None);
            }
        };

        // Remove data before magic
        if magic_pos > 0 {
            for _ in 0..magic_pos {
                self.buffer.remove(0);
            }
        }

        // Check if we have enough data for the header
        if self.buffer.len() < 11 {
            return Ok(None);
        }

        // Parse length from header
        let length = u32::from_le_bytes([
            self.buffer[3],
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
        ]);

        // Check if we have the complete packet
        let total_packet_size = 13 + length as usize;
        if self.buffer.len() < total_packet_size {
            return Ok(None);
        }

        // Extract packet data
        let packet_data: Vec<u8> = self.buffer[..total_packet_size].iter().cloned().collect();

        // Remove processed packet from buffer
        for _ in 0..total_packet_size {
            self.buffer.remove(0);
        }

        // Parse the packet
        match Packet::from_bytes(&packet_data) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(e),
        }
    }

    async fn handle_command<SPI>(
        &mut self,
        packet: Packet,
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        match packet.command {
            Command::Info => self.handle_info_command(flash_driver).await,
            Command::Erase => self.handle_erase_command(packet, flash_driver).await,
            Command::Write => self.handle_write_command(packet, flash_driver).await,
            Command::Read => self.handle_read_command(packet, flash_driver).await,
            Command::Verify => self.handle_verify_command(packet, flash_driver).await,
        }
    }

    async fn handle_info_command<SPI>(&mut self, flash_driver: &mut FlashDriver<SPI>) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        match flash_driver.get_info().await {
            Ok(info) => {
                let mut data = Vec::new();
                data.extend_from_slice(&info.jedec_id.to_le_bytes());
                data.extend_from_slice(&info.total_size.to_le_bytes());
                data.extend_from_slice(&info.page_size.to_le_bytes());
                data.extend_from_slice(&info.sector_size.to_le_bytes());
                Response::new(Status::Success, data)
            }
            Err(e) => {
                error!("Info command failed: {:?}", e);
                Response::new(Status::FlashError, Vec::new())
            }
        }
    }

    async fn handle_erase_command<SPI>(
        &mut self,
        packet: Packet,
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        let size = if packet.data.len() >= 4 {
            u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
        } else {
            FLASH_SECTOR_SIZE as u32 // Default to one sector
        };

        match flash_driver.erase_range(packet.address, size).await {
            Ok(_) => {
                info!("Erase completed: addr=0x{:08X}, size={}", packet.address, size);
                Response::new(Status::Success, Vec::new())
            }
            Err(e) => {
                error!("Erase failed: {:?}", e);
                let status = match e {
                    FlashDriverError::InvalidAddress => Status::InvalidAddress,
                    FlashDriverError::InvalidSize => Status::InvalidAddress,
                    _ => Status::FlashError,
                };
                Response::new(status, Vec::new())
            }
        }
    }

    async fn handle_write_command<SPI>(
        &mut self,
        packet: Packet,
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        if packet.data.is_empty() {
            return Response::new(Status::InvalidAddress, Vec::new());
        }

        match flash_driver.write_data(packet.address, &packet.data).await {
            Ok(_) => {
                info!("Write completed: addr=0x{:08X}, size={}", packet.address, packet.data.len());
                Response::new(Status::Success, Vec::new())
            }
            Err(e) => {
                error!("Write failed: {:?}", e);
                let status = match e {
                    FlashDriverError::InvalidAddress => Status::InvalidAddress,
                    FlashDriverError::InvalidSize => Status::InvalidAddress,
                    _ => Status::FlashError,
                };
                Response::new(status, Vec::new())
            }
        }
    }

    async fn handle_read_command<SPI>(
        &mut self,
        packet: Packet,
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        let size = if packet.data.len() >= 4 {
            u32::from_le_bytes([packet.data[0], packet.data[1], packet.data[2], packet.data[3]])
        } else {
            return Response::new(Status::InvalidAddress, Vec::new());
        };

        if size > MAX_PAYLOAD_SIZE as u32 {
            return Response::new(Status::InvalidAddress, Vec::new());
        }

        let mut buffer = vec![0u8; size as usize];
        match flash_driver.read_data(packet.address, &mut buffer).await {
            Ok(_) => {
                info!("Read completed: addr=0x{:08X}, size={}", packet.address, size);
                Response::new(Status::Success, buffer)
            }
            Err(e) => {
                error!("Read failed: {:?}", e);
                let status = match e {
                    FlashDriverError::InvalidAddress => Status::InvalidAddress,
                    FlashDriverError::InvalidSize => Status::InvalidAddress,
                    _ => Status::FlashError,
                };
                Response::new(status, Vec::new())
            }
        }
    }

    async fn handle_verify_command<SPI>(
        &mut self,
        packet: Packet,
        flash_driver: &mut FlashDriver<SPI>,
    ) -> Response
    where
        SPI: SpiDevice,
        SPI::Error: defmt::Format,
    {
        if packet.data.is_empty() {
            return Response::new(Status::InvalidAddress, Vec::new());
        }

        let mut buffer = vec![0u8; packet.data.len()];
        match flash_driver.read_data(packet.address, &mut buffer).await {
            Ok(_) => {
                if buffer == packet.data {
                    info!("Verify successful: addr=0x{:08X}, size={}", packet.address, packet.data.len());
                    Response::new(Status::Success, Vec::new())
                } else {
                    warn!("Verify failed: data mismatch at addr=0x{:08X}", packet.address);
                    Response::new(Status::CrcError, Vec::new())
                }
            }
            Err(e) => {
                error!("Verify read failed: {:?}", e);
                let status = match e {
                    FlashDriverError::InvalidAddress => Status::InvalidAddress,
                    FlashDriverError::InvalidSize => Status::InvalidAddress,
                    _ => Status::FlashError,
                };
                Response::new(status, Vec::new())
            }
        }
    }
}
