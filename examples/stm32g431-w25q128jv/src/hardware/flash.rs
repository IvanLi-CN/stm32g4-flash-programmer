use embassy_stm32::spi::Spi;
use embassy_stm32::gpio::Output;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;
use heapless::Vec;

use crate::resources::cache::FlashCache;

/// Flash manager with caching support
pub struct FlashManager {
    spi_device: Option<SpiDevice<'static, CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>, Output<'static>>>,
    cache: FlashCache<8>, // 8 cache entries
    initialized: bool,
}

impl FlashManager {
    /// Create new flash manager
    pub fn new() -> Self {
        Self {
            spi_device: None,
            cache: FlashCache::new(),
            initialized: false,
        }
    }

    /// Initialize flash with SPI device
    pub async fn initialize(
        &mut self,
        spi_bus: &'static Mutex<CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>>,
        cs_pin: Output<'static>,
    ) -> Result<(), &'static str> {
        let spi_device = SpiDevice::new(spi_bus, cs_pin);

        // For now, just store the SPI device and mark as initialized
        // Real Flash operations would need proper W25 driver integration
        self.spi_device = Some(spi_device);
        self.initialized = true;

        defmt::info!("Flash SPI device initialized");
        Ok(())
    }

    /// Check if flash is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Read data directly from SPI Flash (W25Q128JV)
    async fn read_from_spi(&mut self, address: u32, length: usize) -> Result<Vec<u8, 1024>, &'static str> {
        if let Some(ref mut spi_device) = self.spi_device {
            let mut buffer = Vec::new();

            // W25Q128JV READ command (0x03) + 24-bit address
            let cmd = [
                0x03,                           // READ command
                (address >> 16) as u8,         // Address high byte
                (address >> 8) as u8,          // Address middle byte
                address as u8,                 // Address low byte
            ];

            // Prepare read buffer
            let mut read_buffer = [0u8; 1024];
            let actual_length = core::cmp::min(length, 1024);

            // Perform SPI transaction
            match spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&cmd),
                embedded_hal_async::spi::Operation::Read(&mut read_buffer[..actual_length]),
            ]).await {
                Ok(_) => {
                    // Copy data to result vector
                    for i in 0..actual_length {
                        buffer.push(read_buffer[i]).map_err(|_| "Buffer full")?;
                    }
                    defmt::debug!("Read {} bytes from SPI Flash at 0x{:08X}", actual_length, address);
                    Ok(buffer)
                },
                Err(_) => {
                    defmt::error!("SPI Flash read failed at address 0x{:08X}", address);
                    Err("SPI Flash read failed")
                }
            }
        } else {
            Err("SPI device not initialized")
        }
    }

    /// Read data from flash with caching
    pub async fn read_data(&mut self, address: u32, length: usize) -> Result<Vec<u8, 2048>, &'static str> {
        if !self.initialized {
            return Err("Flash not initialized");
        }

        // Try to read from cache first
        let mut result = Vec::new();
        let mut remaining_length = length;
        let mut current_address = address;

        // Try to read data in chunks from cache
        while remaining_length > 0 {
            // Try to find a cache entry that contains data starting at current_address
            let mut found_data = false;

            // Check all cache entries to see if any contains data at current_address
            for entry_address in [address, address & !0xFF, (address & !0xFF) + 256, (address & !0xFF) + 512, (address & !0xFF) + 768] {
                if let Some(cached_data) = self.cache.get(entry_address, 1024) {
                    // Calculate offset within this cache entry
                    if current_address >= entry_address && current_address < entry_address + cached_data.len() as u32 {
                        let offset_in_entry = (current_address - entry_address) as usize;
                        let available_in_entry = cached_data.len() - offset_in_entry;
                        let to_read = core::cmp::min(remaining_length, available_in_entry);

                        // Copy data from this cache entry
                        for i in 0..to_read {
                            result.push(cached_data[offset_in_entry + i]).map_err(|_| "Result buffer full")?;
                        }

                        current_address += to_read as u32;
                        remaining_length -= to_read;
                        found_data = true;
                        break;
                    }
                }
            }

            if !found_data {
                // Cache miss - read from SPI Flash and populate cache
                defmt::debug!("Cache miss at address 0x{:08X}, reading from SPI Flash", current_address);

                // Read a larger chunk (256 bytes) to improve cache efficiency
                let chunk_size = core::cmp::min(256, remaining_length);
                let chunk_address = current_address & !0xFF; // Align to 256-byte boundary

                match self.read_from_spi(chunk_address, 256).await {
                    Ok(spi_data) => {
                        // Store in cache
                        if let Err(e) = self.cache.put(chunk_address, &spi_data) {
                            defmt::warn!("Failed to cache data: {}", e);
                        }

                        // Extract the requested data from the chunk
                        let offset_in_chunk = (current_address - chunk_address) as usize;
                        let available_in_chunk = spi_data.len() - offset_in_chunk;
                        let to_read = core::cmp::min(remaining_length, available_in_chunk);

                        for i in 0..to_read {
                            result.push(spi_data[offset_in_chunk + i]).map_err(|_| "Result buffer full")?;
                        }

                        current_address += to_read as u32;
                        remaining_length -= to_read;
                    },
                    Err(e) => {
                        defmt::error!("Failed to read from SPI Flash at 0x{:08X}: {}", current_address, e);
                        return Err("SPI Flash read failed");
                    }
                }
            }
        }

        defmt::debug!("Read {} bytes from Flash cache starting at address 0x{:08X}", length, address);
        Ok(result)
    }

    /// Read a small chunk of data (for headers, etc.)
    pub async fn read_chunk(&mut self, address: u32, length: usize) -> Result<Vec<u8, 256>, &'static str> {
        if length > 256 {
            return Err("Chunk too large");
        }

        let data = self.read_data(address, length).await?;
        let mut chunk = Vec::new();

        for &byte in &data[..core::cmp::min(length, data.len())] {
            chunk.push(byte).map_err(|_| "Chunk buffer full")?;
        }

        Ok(chunk)
    }

    // Write method removed - no fonts stored in firmware

    /// Get flash information (simplified for now)
    pub async fn get_flash_info(&mut self) -> Result<FlashInfo, &'static str> {
        if !self.initialized {
            return Err("Flash not initialized");
        }

        // Return dummy info for W25Q128JV (would need proper driver integration)
        Ok(FlashInfo {
            jedec_id: 0xEF4018, // W25Q128JV JEDEC ID
            total_size: 16 * 1024 * 1024, // 16MB for W25Q128JV
            page_size: 256,
            sector_size: 4096,
            block_size: 65536,
        })
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> crate::resources::cache::CacheStats {
        self.cache.stats()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Read JEDEC ID from Flash chip to verify SPI communication
    pub async fn read_jedec_id(&mut self) -> Result<[u8; 3], &'static str> {
        if let Some(ref mut spi_device) = self.spi_device {
            // JEDEC ID command: 0x9F
            let cmd_buf = [0x9F_u8]; // Command to read JEDEC ID
            let mut id_buf = [0_u8; 3]; // 3 bytes: Manufacturer ID + Device ID

            match spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&cmd_buf),
                embedded_hal_async::spi::Operation::Read(&mut id_buf),
            ]).await {
                Ok(_) => {
                    defmt::debug!("📥 JEDEC ID: {:02X} {:02X} {:02X}", id_buf[0], id_buf[1], id_buf[2]);
                    Ok(id_buf)
                },
                Err(_) => {
                    defmt::error!("❌ Failed to read JEDEC ID via SPI");
                    Err("SPI transaction failed")
                }
            }
        } else {
            Err("SPI device not initialized")
        }
    }

    /// Simple, memory-safe Flash read for font data (using same method as firmware)
    pub async fn read_data_simple(&mut self, address: u32, length: usize) -> Result<heapless::Vec<u8, 64>, &'static str> {
        if let Some(ref mut spi_device) = self.spi_device {
            // Limit read size to prevent memory issues
            let safe_length = length.min(64);

            // Read command: 0x03 (Read Data) - same as firmware
            let cmd_buf = [
                0x03, // CMD_READ_DATA
                (address >> 16) as u8,
                (address >> 8) as u8,
                address as u8,
            ];

            defmt::debug!("🔍 SPI Read: addr=0x{:08X}, len={}", address, safe_length);
            defmt::debug!("📤 SPI CMD: {:?}", cmd_buf);

            // Create exact-size buffer like firmware does
            let mut read_buf = heapless::Vec::<u8, 64>::new();
            read_buf.resize(safe_length, 0).map_err(|_| "Buffer resize failed")?;

            // Use the SAME transaction method as firmware
            match spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&cmd_buf),
                embedded_hal_async::spi::Operation::Read(&mut read_buf),  // Direct buffer, no slicing
            ]).await {
                Ok(_) => {
                    defmt::debug!("📥 SPI Data: {:?}", &read_buf[..read_buf.len().min(8)]);
                    defmt::debug!("✅ Result: {:?}", &read_buf[..read_buf.len().min(8)]);
                    Ok(read_buf)
                },
                Err(_) => {
                    defmt::error!("❌ SPI transaction failed for simple read at 0x{:08X}", address);
                    Err("SPI transaction failed")
                }
            }
        } else {
            Err("SPI device not initialized")
        }
    }
}

/// Flash information structure
#[derive(Debug, Clone)]
pub struct FlashInfo {
    pub jedec_id: u32,
    pub total_size: u32,
    pub page_size: u32,
    pub sector_size: u32,
    pub block_size: u32,
}
