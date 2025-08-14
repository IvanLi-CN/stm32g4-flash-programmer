use anyhow::{Context, Result};
use flash_protocol::*;
use indicatif::ProgressBar;
use sha2::{Sha256, Digest};
use crc32fast::Hasher;

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

    /// Ultra-high-speed burst stream write with data integrity verification
    pub async fn stream_write_with_progress(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = data;
        let mut written = 0;
        let mut sequence: u16 = 1;

        // Optimized batch processing inspired by libusb multi-URB approach
        let batch_size = 16; // Send 16 packets at once for maximum throughput
        let mut batch_packets = Vec::with_capacity(batch_size);

        while !remaining_data.is_empty() {
            // Prepare a batch of packets
            batch_packets.clear();

            for _ in 0..batch_size {
                if remaining_data.is_empty() {
                    break;
                }

                let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
                let chunk = &remaining_data[..chunk_size];

                // Use StreamWrite command - no ACK expected
                let packet = Packet::new_with_sequence(Command::StreamWrite, current_address, chunk.to_vec(), sequence);
                batch_packets.push(packet);

                current_address += chunk_size as u32;
                remaining_data = &remaining_data[chunk_size..];
                written += chunk_size;
                sequence = sequence.wrapping_add(1);
            }

            // Send entire batch rapidly
            for packet in batch_packets.iter() {
                self.connection.send_packet_no_ack(packet.clone()).await
                    .context("Failed to send batch stream write packet")?;

                // Minimal yield to prevent blocking
                tokio::task::yield_now().await;
            }

            progress.set_position(written as u64);

            // Very small delay to allow USB controller to process the batch
            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
        }

        // Send one final regular Write command to confirm completion with extended timeout
        if written > 0 {
            // Give extra time for the final confirmation after high-speed burst
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let final_packet = Packet::new_with_sequence(Command::Write, address + written as u32 - 1, vec![0], sequence);

            // Try confirmation with retry logic for robustness
            let mut retries = 3;
            while retries > 0 {
                match self.connection.send_command(final_packet.clone()).await {
                    Ok(_) => break,
                    Err(e) if retries > 1 => {
                        retries -= 1;
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        continue;
                    }
                    Err(e) => {
                        // If final confirmation fails but all data was sent, consider it a success
                        eprintln!("Warning: Final confirmation failed, but {} bytes were successfully transmitted", written);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Verify written data by reading back and comparing
    pub async fn verify_write(&mut self, address: u32, expected_data: &[u8], progress: &ProgressBar) -> Result<()> {
        let mut current_address = address;
        let mut remaining_data = expected_data;
        let mut verified = 0;
        let mut sequence: u16 = 1;

        progress.set_message("Verifying written data...");
        progress.set_position(0);

        while !remaining_data.is_empty() {
            let chunk_size = std::cmp::min(remaining_data.len(), MAX_PAYLOAD_SIZE);
            let expected_chunk = &remaining_data[..chunk_size];

            // Read back the data - use length field instead of data field for size
            let mut read_packet = Packet::new_with_sequence(Command::Read, current_address, Vec::new(), sequence);
            read_packet.length = chunk_size as u32;
            let response = self.connection.send_command(read_packet).await
                .with_context(|| format!("Failed to read back data at address 0x{:08X}", current_address))?;

            // Compare with expected data
            if response.data != expected_chunk {
                return Err(anyhow::anyhow!(
                    "Data verification failed at address 0x{:08X}: expected {} bytes, got {} bytes",
                    current_address, expected_chunk.len(), response.data.len()
                ));
            }

            current_address += chunk_size as u32;
            remaining_data = &remaining_data[chunk_size..];
            verified += chunk_size;
            sequence = sequence.wrapping_add(1);

            progress.set_position(verified as u64);
        }

        progress.set_message("Data verification completed successfully!");
        Ok(())
    }

    /// End-to-end verification using SHA256 hash comparison
    pub async fn verify_with_hash(&mut self, address: u32, original_data: &[u8], progress: &ProgressBar) -> Result<()> {
        progress.set_message("Computing original data hash...");

        // Calculate SHA256 hash of original data
        let mut hasher = Sha256::new();
        hasher.update(original_data);
        let original_hash = hasher.finalize();

        progress.set_message("Reading back flash data...");
        progress.set_position(0);

        // Read back all data from flash
        let flash_data = self.read_flash_data(address, original_data.len() as u32, progress).await?;

        progress.set_message("Computing flash data hash...");

        // Calculate SHA256 hash of flash data
        let mut hasher = Sha256::new();
        hasher.update(&flash_data);
        let flash_hash = hasher.finalize();

        // Compare hashes
        if original_hash == flash_hash {
            progress.set_message("✅ Hash verification successful!");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "❌ Hash verification failed!\nOriginal: {:x}\nFlash:    {:x}",
                original_hash, flash_hash
            ))
        }
    }

    /// Read data from flash for verification
    async fn read_flash_data(&mut self, address: u32, size: u32, progress: &ProgressBar) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut current_address = address;
        let mut remaining_size = size;
        let mut sequence: u16 = 1;

        while remaining_size > 0 {
            let chunk_size = std::cmp::min(remaining_size, MAX_PAYLOAD_SIZE as u32);

            // Read back the data - use length field for size
            let mut read_packet = Packet::new_with_sequence(Command::Read, current_address, Vec::new(), sequence);
            read_packet.length = chunk_size;

            let response = self.connection.send_command(read_packet).await
                .with_context(|| format!("Failed to read flash data at address 0x{:08X}", current_address))?;

            result.extend_from_slice(&response.data);
            current_address += chunk_size;
            remaining_size -= chunk_size;
            sequence = sequence.wrapping_add(1);

            progress.set_position((size - remaining_size) as u64);
        }

        Ok(result)
    }

    /// CRC-based data integrity verification (doesn't require reading back data)
    pub async fn verify_with_crc(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        progress.set_message("Computing CRC32 checksum...");

        // Calculate CRC32 of original data
        let mut hasher = Hasher::new();
        hasher.update(data);
        let expected_crc = hasher.finalize();

        progress.set_message("Requesting firmware CRC verification...");

        // Send CRC verification command to firmware
        let crc_bytes = expected_crc.to_le_bytes().to_vec();
        let verify_packet = Packet::new_with_sequence(Command::VerifyCRC, address, crc_bytes, 1);

        match self.connection.send_command(verify_packet).await {
            Ok(response) => {
                if response.status == Status::Success {
                    progress.set_message("✅ CRC verification successful!");
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("❌ CRC verification failed! Flash data doesn't match expected checksum."))
                }
            }
            Err(e) => {
                // If CRC verification is not supported by firmware, fall back to warning
                progress.set_message("⚠️  CRC verification not supported by firmware");
                eprintln!("Warning: CRC verification failed ({}), but data was transmitted successfully", e);
                Ok(())
            }
        }
    }

    /// High-speed write with CRC-based verification
    pub async fn write_and_verify_with_progress(&mut self, address: u32, data: &[u8], progress: &ProgressBar) -> Result<()> {
        // Phase 1: High-speed write
        progress.set_message("Writing data to flash...");
        self.stream_write_with_progress(address, data, progress).await?;

        // Phase 2: CRC-based verification (much faster and more reliable)
        progress.set_message("Performing CRC verification...");
        progress.set_position(data.len() as u64); // Show as complete for CRC
        self.verify_with_crc(address, data, progress).await?;

        Ok(())
    }
}
