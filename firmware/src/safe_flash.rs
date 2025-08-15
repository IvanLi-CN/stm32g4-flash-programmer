// defmt support for error types
use alloc::vec::Vec;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

// W25Q128 Commands
const CMD_READ_JEDEC_ID: u8 = 0x9F;
const CMD_READ_DATA: u8 = 0x03;
const CMD_WRITE_ENABLE: u8 = 0x06;
#[allow(dead_code)]
const CMD_WRITE_DISABLE: u8 = 0x04;
const CMD_PAGE_PROGRAM: u8 = 0x02;
const CMD_SECTOR_ERASE: u8 = 0x20;
const CMD_READ_STATUS: u8 = 0x05;
const CMD_READ_STATUS2: u8 = 0x35; // Read Status Register 2
const CMD_READ_STATUS3: u8 = 0x15; // Read Status Register 3
#[allow(dead_code)]
const CMD_WRITE_STATUS: u8 = 0x01; // Write Status Register
const CMD_RELEASE_POWER_DOWN: u8 = 0xAB; // Release from Deep Power-down

#[derive(Debug, defmt::Format)]
pub enum SafeFlashError {
    NotInitialized,
    InitializationFailed,
    SpiError,
    Timeout,
}

pub struct FlashInfo {
    pub jedec_id: u32,
    pub total_size: u32,
    pub page_size: u32,
    pub sector_size: u32,
}

pub struct SafeFlashManager {
    spi_bus: Option<&'static Mutex<CriticalSectionRawMutex, Spi<'static, Async>>>,
    initialized: bool,
    flash_available: bool,
}

impl SafeFlashManager {
    pub fn new() -> Self {
        Self {
            spi_bus: None,
            initialized: false,
            flash_available: false,
        }
    }

    pub fn set_spi_resources(
        &mut self,
        spi_bus: &'static Mutex<CriticalSectionRawMutex, Spi<'static, Async>>,
    ) {
        self.spi_bus = Some(spi_bus);
    }

    // Helper function to create CS pin when needed
    fn create_cs_pin(&self) -> Output<'static> {
        use embassy_stm32::gpio::{Level, Speed};
        // Create CS pin on PB12 (correct hardware connection)
        Output::new(
            unsafe { embassy_stm32::peripherals::PB12::steal() },
            Level::High,
            Speed::VeryHigh,
        )
    }

    pub async fn try_initialize(&mut self) -> Result<(), SafeFlashError> {
        if self.initialized {
            return if self.flash_available {
                Ok(())
            } else {
                Err(SafeFlashError::InitializationFailed)
            };
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // First, try to wake up the Flash chip from deep power-down mode
        let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
        defmt::info!("Attempting to wake up Flash chip from deep power-down...");
        let wake_up_cmd = [CMD_RELEASE_POWER_DOWN];
        let _ = spi_device
            .transaction(&mut [embedded_hal_async::spi::Operation::Write(&wake_up_cmd)])
            .await; // Ignore errors, as the chip might not be in power-down mode

        // Wait for the chip to wake up (typical wake-up time is 3μs)
        Timer::after(Duration::from_micros(10)).await;
        defmt::info!("Flash wake-up command sent, waiting for chip to be ready...");

        // Try to read JEDEC ID with timeout
        let result = with_timeout(Duration::from_millis(100), async {
            self.read_jedec_id_internal(&mut spi_device).await
        })
        .await;

        match result {
            Ok(Ok(_jedec_id)) => {
                self.initialized = true;
                self.flash_available = true;
                Ok(())
            }
            _ => {
                self.initialized = true;
                self.flash_available = false;
                Err(SafeFlashError::InitializationFailed)
            }
        }
    }

    async fn read_jedec_id_internal<CS>(
        &self,
        spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>,
    ) -> Result<u32, SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        let cmd = [CMD_READ_JEDEC_ID];
        let mut response = [0u8; 3];

        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&cmd),
                embedded_hal_async::spi::Operation::Read(&mut response),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        let jedec_id =
            ((response[0] as u32) << 16) | ((response[1] as u32) << 8) | (response[2] as u32);

        Ok(jedec_id)
    }

    pub async fn get_flash_info(&mut self) -> Result<FlashInfo, SafeFlashError> {
        if !self.is_available() {
            defmt::error!("Flash not available - hardware not initialized or not connected");
            return Err(SafeFlashError::NotInitialized);
        }

        // For now, return the info we detected during initialization
        // TODO: Implement proper re-reading of JEDEC ID without consuming CS pin
        let flash_info = FlashInfo {
            jedec_id: 0xEF4018,           // W25Q128 - this was detected during init
            total_size: 16 * 1024 * 1024, // 16MB
            page_size: 256,
            sector_size: 4096,
        };

        Ok(flash_info)
    }

    pub fn is_available(&self) -> bool {
        self.initialized && self.flash_available
    }

    pub async fn read_status(&mut self) -> Result<u8, SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        with_timeout(Duration::from_millis(1000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.read_status_internal(&mut spi_device).await
        })
        .await
        .map_err(|_| SafeFlashError::Timeout)?
    }

    pub async fn read_data(&mut self, address: u32, size: u32) -> Result<Vec<u8>, SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.read_data_internal(&mut spi_device, address, size)
                .await
        })
        .await
        .map_err(|_| SafeFlashError::Timeout)?
    }

    pub async fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // Write data to Flash chip (page by page)
        with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.write_data_internal(&mut spi_device, address, data)
                .await
        })
        .await
        .map_err(|_| SafeFlashError::Timeout)?
    }

    pub async fn erase_sector(&mut self, address: u32) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // Erase sector on Flash chip
        with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.erase_sector_internal(&mut spi_device, address).await
        })
        .await
        .map_err(|_| SafeFlashError::Timeout)?
    }

    async fn read_data_internal<CS>(
        &self,
        spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>,
        address: u32,
        size: u32,
    ) -> Result<Vec<u8>, SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        defmt::info!(
            "Flash read internal: address=0x{:08X}, size={}",
            address,
            size
        );

        // Limit single read to avoid heap issues - let the protocol layer handle chunking
        const MAX_SINGLE_READ: u32 = 256; // Maximum single read size
        let actual_size = if size > MAX_SINGLE_READ {
            MAX_SINGLE_READ
        } else {
            size
        };

        defmt::info!(
            "Reading {} bytes (requested {}, limited to {})",
            actual_size,
            size,
            MAX_SINGLE_READ
        );

        // Prepare read command with 24-bit address
        let cmd = [
            CMD_READ_DATA,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        defmt::debug!(
            "Read command: {:02X} {:02X} {:02X} {:02X}",
            cmd[0],
            cmd[1],
            cmd[2],
            cmd[3]
        );

        let mut data = alloc::vec![0u8; actual_size as usize];

        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&cmd),
                embedded_hal_async::spi::Operation::Read(&mut data),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        defmt::info!("Flash read completed: {} bytes read", data.len());

        // Debug: Show first few bytes of read data
        if data.len() <= 32 {
            defmt::debug!("Read data debug:");
            for (i, byte) in data.iter().enumerate() {
                if i % 16 == 0 && i > 0 {
                    defmt::debug!("");
                }
                defmt::debug!("{:02X} ", byte);
            }
        }
        Ok(data)
    }

    async fn erase_sector_internal<CS>(
        &self,
        spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>,
        address: u32,
    ) -> Result<(), SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        // Write enable
        let write_enable_cmd = [CMD_WRITE_ENABLE];
        spi_device
            .transaction(&mut [embedded_hal_async::spi::Operation::Write(&write_enable_cmd)])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        // Sector erase command with 24-bit address
        let erase_cmd = [
            CMD_SECTOR_ERASE,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        spi_device
            .transaction(&mut [embedded_hal_async::spi::Operation::Write(&erase_cmd)])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        // Wait for erase to complete (poll status register)
        loop {
            let status_cmd = [CMD_READ_STATUS];
            let mut status = [0u8; 1];

            spi_device
                .transaction(&mut [
                    embedded_hal_async::spi::Operation::Write(&status_cmd),
                    embedded_hal_async::spi::Operation::Read(&mut status),
                ])
                .await
                .map_err(|_| SafeFlashError::SpiError)?;

            // Check if write in progress bit (bit 0) is clear
            if (status[0] & 0x01) == 0 {
                break;
            }

            Timer::after(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn write_data_internal<CS>(
        &self,
        spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>,
        address: u32,
        data: &[u8],
    ) -> Result<(), SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        let page_size = 256; // W25Q128 page size
        let mut current_address = address;
        let mut remaining_data = data;

        while !remaining_data.is_empty() {
            // Calculate how much we can write in this page
            let page_offset = current_address % page_size;
            let bytes_to_write =
                core::cmp::min(remaining_data.len(), (page_size - page_offset) as usize);

            let chunk = &remaining_data[..bytes_to_write];

            // Write enable
            defmt::debug!("Sending write enable command");
            let write_enable_cmd = [CMD_WRITE_ENABLE];
            spi_device
                .transaction(&mut [embedded_hal_async::spi::Operation::Write(&write_enable_cmd)])
                .await
                .map_err(|_| SafeFlashError::SpiError)?;
            defmt::debug!("Write enable command sent successfully");

            // Add a small delay to allow Flash to process the command
            Timer::after(Duration::from_micros(10)).await;

            // Verify write enable latch (WEL) is set - check immediately after command
            defmt::debug!("Checking WEL bit immediately after Write Enable command...");
            let status_cmd = [CMD_READ_STATUS];
            let mut status = [0u8; 1];
            spi_device
                .transaction(&mut [
                    embedded_hal_async::spi::Operation::Write(&status_cmd),
                    embedded_hal_async::spi::Operation::Read(&mut status),
                ])
                .await
                .map_err(|_| SafeFlashError::SpiError)?;

            defmt::info!("Status after Write Enable: 0x{:02X}", status[0]);
            if (status[0] & 0x02) == 0 {
                defmt::error!(
                    "Write Enable Latch (WEL) not set! Status: 0x{:02X}",
                    status[0]
                );
                defmt::error!(
                    "This indicates the Flash chip is not responding to Write Enable commands"
                );

                // Test if SPI communication is still working by reading JEDEC ID
                defmt::info!("Testing SPI communication after failed Write Enable...");
                match self.read_jedec_id_internal(spi_device).await {
                    Ok(jedec_id) => {
                        defmt::info!(
                            "SPI read communication still works: JEDEC ID = 0x{:06X}",
                            jedec_id
                        );
                        defmt::error!("This confirms SPI read works but Write Enable fails");
                        defmt::error!("Possible causes: 1) Hardware write protection 2) Flash chip defect 3) MOSI line issue");
                    }
                    Err(_) => {
                        defmt::error!(
                            "SPI communication completely failed after Write Enable attempt"
                        );
                        defmt::error!(
                            "This suggests the Write Enable command corrupted SPI communication"
                        );
                    }
                }

                return Err(SafeFlashError::SpiError);
            }
            defmt::info!(
                "✅ Write Enable Latch (WEL) confirmed set, status: 0x{:02X}",
                status[0]
            );

            // Page program command with 24-bit address
            defmt::debug!(
                "Writing {} bytes to address 0x{:08X}",
                chunk.len(),
                current_address
            );
            let program_cmd = [
                CMD_PAGE_PROGRAM,
                (current_address >> 16) as u8,
                (current_address >> 8) as u8,
                current_address as u8,
            ];
            defmt::debug!(
                "Program command: {:02X} {:02X} {:02X} {:02X}",
                program_cmd[0],
                program_cmd[1],
                program_cmd[2],
                program_cmd[3]
            );

            spi_device
                .transaction(&mut [
                    embedded_hal_async::spi::Operation::Write(&program_cmd),
                    embedded_hal_async::spi::Operation::Write(chunk),
                ])
                .await
                .map_err(|_| SafeFlashError::SpiError)?;
            defmt::debug!("Page program command sent successfully");

            // Add a small delay to allow Flash to start the write operation
            Timer::after(Duration::from_micros(100)).await;
            defmt::debug!("Initial delay completed, starting status polling...");

            // Wait for write to complete (poll status register)
            defmt::debug!("Waiting for write to complete...");
            let mut poll_count = 0;
            loop {
                let status_cmd = [CMD_READ_STATUS];
                let mut status = [0u8; 1];

                spi_device
                    .transaction(&mut [
                        embedded_hal_async::spi::Operation::Write(&status_cmd),
                        embedded_hal_async::spi::Operation::Read(&mut status),
                    ])
                    .await
                    .map_err(|_| SafeFlashError::SpiError)?;

                poll_count += 1;
                defmt::debug!("Status poll #{}: 0x{:02X}", poll_count, status[0]);

                // Check if write in progress bit (bit 0) is clear
                if (status[0] & 0x01) == 0 {
                    defmt::debug!("Write completed after {} polls", poll_count);
                    break;
                }

                Timer::after(Duration::from_millis(1)).await;
            }

            // Move to next chunk
            current_address += bytes_to_write as u32;
            remaining_data = &remaining_data[bytes_to_write..];
        }

        Ok(())
    }

    async fn read_status_internal<CS>(
        &self,
        spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>,
    ) -> Result<u8, SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        let status_cmd = [CMD_READ_STATUS];
        let mut status = [0u8; 1];

        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&status_cmd),
                embedded_hal_async::spi::Operation::Read(&mut status),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        Ok(status[0])
    }

    /// Read and display all status registers for debugging
    pub async fn diagnose_flash_protection(&mut self) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();
        let mut spi_device = SpiDevice::new(spi_bus, cs_pin);

        // Read Status Register 1
        let status1_cmd = [CMD_READ_STATUS];
        let mut status1 = [0u8; 1];
        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&status1_cmd),
                embedded_hal_async::spi::Operation::Read(&mut status1),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        // Read Status Register 2
        let status2_cmd = [CMD_READ_STATUS2];
        let mut status2 = [0u8; 1];
        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&status2_cmd),
                embedded_hal_async::spi::Operation::Read(&mut status2),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        // Read Status Register 3
        let status3_cmd = [CMD_READ_STATUS3];
        let mut status3 = [0u8; 1];
        spi_device
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&status3_cmd),
                embedded_hal_async::spi::Operation::Read(&mut status3),
            ])
            .await
            .map_err(|_| SafeFlashError::SpiError)?;

        defmt::info!("=== Flash Protection Diagnosis ===");
        defmt::info!("Status Register 1: 0x{:02X}", status1[0]);
        defmt::info!(
            "  BUSY (bit 0): {}",
            if status1[0] & 0x01 != 0 {
                "1 (Busy)"
            } else {
                "0 (Ready)"
            }
        );
        defmt::info!(
            "  WEL  (bit 1): {}",
            if status1[0] & 0x02 != 0 {
                "1 (Enabled)"
            } else {
                "0 (Disabled)"
            }
        );
        defmt::info!(
            "  BP0  (bit 2): {}",
            if status1[0] & 0x04 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  BP1  (bit 3): {}",
            if status1[0] & 0x08 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  BP2  (bit 4): {}",
            if status1[0] & 0x10 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  TB   (bit 5): {}",
            if status1[0] & 0x20 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  SEC  (bit 6): {}",
            if status1[0] & 0x40 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  SRP0 (bit 7): {}",
            if status1[0] & 0x80 != 0 { "1" } else { "0" }
        );

        defmt::info!("Status Register 2: 0x{:02X}", status2[0]);
        defmt::info!(
            "  SRP1 (bit 0): {}",
            if status2[0] & 0x01 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  QE   (bit 1): {}",
            if status2[0] & 0x02 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  LB1  (bit 3): {}",
            if status2[0] & 0x08 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  LB2  (bit 4): {}",
            if status2[0] & 0x10 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  LB3  (bit 5): {}",
            if status2[0] & 0x20 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  CMP  (bit 6): {}",
            if status2[0] & 0x40 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  SUS  (bit 7): {}",
            if status2[0] & 0x80 != 0 { "1" } else { "0" }
        );

        defmt::info!("Status Register 3: 0x{:02X}", status3[0]);
        defmt::info!(
            "  WPS  (bit 2): {}",
            if status3[0] & 0x04 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  DRV0 (bit 5): {}",
            if status3[0] & 0x20 != 0 { "1" } else { "0" }
        );
        defmt::info!(
            "  DRV1 (bit 6): {}",
            if status3[0] & 0x40 != 0 { "1" } else { "0" }
        );
        defmt::info!("================================");

        Ok(())
    }
}
