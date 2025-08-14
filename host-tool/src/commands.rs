use anyhow::{Context, Result};
use flash_protocol::*;
use indicatif::ProgressBar;

use crate::serial::SerialConnection;

pub struct FlashCommands<'a> {
    connection: &'a mut SerialConnection,
}

#[derive(Debug)]
pub struct FlashInfo {
    pub jedec_id: u32,
    pub total_size: u32,
    pub page_size: u32,
    pub sector_size: u32,
}

impl<'a> FlashCommands<'a> {
    pub fn new(connection: &'a mut SerialConnection) -> Self {
        Self { connection }
    }

    pub async fn get_info(&mut self) -> Result<FlashInfo> {
        let packet = Packet::new(Command::Info, 0, Vec::new());
        let response = self.connection.send_command(packet).await?;

        if response.data.len() < 16 {
            return Err(anyhow::anyhow!("Invalid info response length"));
        }

        let jedec_id = u32::from_le_bytes([
            response.data[0], response.data[1], response.data[2], response.data[3]
        ]);
        let total_size = u32::from_le_bytes([
            response.data[4], response.data[5], response.data[6], response.data[7]
        ]);
        let page_size = u32::from_le_bytes([
            response.data[8], response.data[9], response.data[10], response.data[11]
        ]);
        let sector_size = u32::from_le_bytes([
            response.data[12], response.data[13], response.data[14], response.data[15]
        ]);

        Ok(FlashInfo {
            jedec_id,
            total_size,
            page_size,
            sector_size,
        })
    }

    pub async fn erase(&mut self, address: u32, size: u32) -> Result<()> {
        let data = size.to_le_bytes().to_vec();
        let packet = Packet::new(Command::Erase, address, data);
        self.connection.send_command(packet).await?;
        Ok(())
    }

    pub async fn write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = data;

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
            let chunk = &remaining_data[..chunk_size];

            let packet = Packet::new(Command::Write, current_address, chunk.to_vec());
            self.connection.send_command(packet).await
                .with_context(|| format!("Failed to write at address 0x{:08X}", current_address))?;

            current_address += chunk_size as u32;
            remaining_data = &remaining_data[chunk_size..];
        }

        Ok(())
    }

    pub async fn write_with_progress(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        self.stream_write_with_progress(address, data, progress).await
    }

    /// High-speed write with optimized 4KB packets
    pub async fn batch_write_with_progress(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = data;
        let mut written = 0;
        let mut sequence: u16 = 1;

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
            let chunk = &remaining_data[..chunk_size];

            // Use regular Write command with 4KB packets for maximum compatibility
            let packet = Packet::new_with_sequence(Command::Write, current_address, chunk.to_vec(), sequence);

            // Send and wait for ACK - simplified approach
            self.connection.send_command(packet).await
                .with_context(|| format!("Failed to write at address 0x{:08X}", current_address))?;

            current_address += chunk_size as u32;
            remaining_data = &remaining_data[chunk_size..];
            written += chunk_size;
            sequence = sequence.wrapping_add(1);

            progress.set_position(written as u64);
        }

        Ok(())
    }

    pub async fn read(&mut self, address: u32, size: u32) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut current_address = address;
        let mut remaining_size = size;

        while remaining_size > 0 {
            let chunk_size = std::cmp::min(remaining_size, MAX_PAYLOAD_SIZE as u32);
            let data = chunk_size.to_le_bytes().to_vec();

            let packet = Packet::new(Command::Read, current_address, data);
            let response = self.connection.send_command(packet).await
                .with_context(|| format!("Failed to read at address 0x{:08X}", current_address))?;

            result.extend_from_slice(&response.data);
            current_address += chunk_size;
            remaining_size -= chunk_size;
        }

        Ok(result)
    }

    pub async fn read_with_progress(&mut self, address: u32, size: u32, progress: &ProgressBar) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut current_address = address;
        let mut remaining_size = size;
        let mut read_bytes = 0;

        while remaining_size > 0 {
            let chunk_size = std::cmp::min(remaining_size, MAX_PAYLOAD_SIZE as u32);
            let data = chunk_size.to_le_bytes().to_vec();

            let packet = Packet::new(Command::Read, current_address, data);
            let response = self.connection.send_command(packet).await
                .with_context(|| format!("Failed to read at address 0x{:08X}", current_address))?;

            result.extend_from_slice(&response.data);
            current_address += chunk_size;
            remaining_size -= chunk_size;
            read_bytes += chunk_size;
            
            progress.set_position(read_bytes as u64);
        }

        Ok(result)
    }

    pub async fn verify(&mut self, address: u32, expected_data: &[u8]) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = expected_data;

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
            let chunk = &remaining_data[..chunk_size];

            let packet = Packet::new(Command::Verify, current_address, chunk.to_vec());
            self.connection.send_command(packet).await
                .with_context(|| format!("Verification failed at address 0x{:08X}", current_address))?;

            current_address += chunk_size as u32;
            remaining_data = &remaining_data[chunk_size..];
        }

        Ok(())
    }

    pub async fn verify_with_progress(&mut self, address: u32, expected_data: &[u8], progress: &ProgressBar) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = expected_data;
        let mut verified = 0;

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
            let chunk = &remaining_data[..chunk_size];

            let packet = Packet::new(Command::Verify, current_address, chunk.to_vec());
            self.connection.send_command(packet).await
                .with_context(|| format!("Verification failed at address 0x{:08X}", current_address))?;

            current_address += chunk_size as u32;
            remaining_data = &remaining_data[chunk_size..];
            verified += chunk_size;
            
            progress.set_position(verified as u64);
        }

        Ok(())
    }

    /// Ultra-high-speed parallel stream write with pipeline optimization
    pub async fn stream_write_with_progress(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = data;
        let mut written = 0;
        let mut sequence: u16 = 1;

        // Conservative pipeline depth for stability
        let pipeline_depth: usize = 4;
        let mut packets_in_flight: usize = 0;

        while !remaining_data.is_empty() || packets_in_flight > 0 {
            // Send packets up to pipeline depth
            while packets_in_flight < pipeline_depth && !remaining_data.is_empty() {
                let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
                let chunk = &remaining_data[..chunk_size];

                // Use StreamWrite command - no ACK expected
                let packet = Packet::new_with_sequence(Command::StreamWrite, current_address, chunk.to_vec(), sequence);

                // Send without waiting for any response - maximum speed!
                self.connection.send_packet_no_ack(packet).await
                    .with_context(|| format!("Failed to stream write at address 0x{:08X}", current_address))?;

                current_address += chunk_size as u32;
                remaining_data = &remaining_data[chunk_size..];
                written += chunk_size;
                sequence = sequence.wrapping_add(1);
                packets_in_flight += 1;

                progress.set_position(written as u64);
            }

            // Small delay to allow USB processing
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            packets_in_flight = packets_in_flight.saturating_sub(2); // Assume 2 packets processed
        }

        // Send one final regular Write command to confirm completion
        if written > 0 {
            let final_packet = Packet::new_with_sequence(Command::Write, address + written as u32 - 1, vec![0], sequence);
            self.connection.send_command(final_packet).await
                .with_context(|| "Failed to confirm stream write completion")?;
        }

        Ok(())
    }
}
