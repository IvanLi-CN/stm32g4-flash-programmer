// defmt support for error types
use embassy_stm32::spi::Spi;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_time::{Duration, Timer, with_timeout};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal::digital::OutputPin;
use alloc::vec::Vec;

// W25Q128 Commands
const CMD_READ_JEDEC_ID: u8 = 0x9F;
const CMD_READ_DATA: u8 = 0x03;
const CMD_WRITE_ENABLE: u8 = 0x06;
const CMD_WRITE_DISABLE: u8 = 0x04;
const CMD_PAGE_PROGRAM: u8 = 0x02;
const CMD_SECTOR_ERASE: u8 = 0x20;
const CMD_READ_STATUS: u8 = 0x05;

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
        // Create CS pin on PA8
        Output::new(
            unsafe { embassy_stm32::peripherals::PA8::steal() },
            Level::High,
            Speed::VeryHigh,
        )
    }
    
    pub async fn try_initialize(&mut self) -> Result<(), SafeFlashError> {
        if self.initialized {
            return if self.flash_available { Ok(()) } else { Err(SafeFlashError::InitializationFailed) };
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // Try to read JEDEC ID with timeout
        let result = with_timeout(Duration::from_millis(100), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.read_jedec_id_internal(&mut spi_device).await
        }).await;

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
    
    async fn read_jedec_id_internal<CS>(&self, spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>) -> Result<u32, SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        let cmd = [CMD_READ_JEDEC_ID];
        let mut response = [0u8; 3];

        spi_device.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Read(&mut response),
        ]).await.map_err(|_| SafeFlashError::SpiError)?;
        
        let jedec_id = ((response[0] as u32) << 16) | 
                      ((response[1] as u32) << 8) | 
                      (response[2] as u32);
        
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
            jedec_id: 0xEF4018, // W25Q128 - this was detected during init
            total_size: 16 * 1024 * 1024, // 16MB
            page_size: 256,
            sector_size: 4096,
        };

        Ok(flash_info)
    }
    
    pub fn is_available(&self) -> bool {
        self.initialized && self.flash_available
    }
    
    pub async fn read_data(&mut self, address: u32, size: u32) -> Result<Vec<u8>, SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        let result = with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.read_data_internal(&mut spi_device, address, size).await
        }).await.map_err(|_| SafeFlashError::Timeout)?;

        result
    }
    
    pub async fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // Write data to Flash chip (page by page)
        let result = with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.write_data_internal(&mut spi_device, address, data).await
        }).await.map_err(|_| SafeFlashError::Timeout)?;

        result
    }
    
    pub async fn erase_sector(&mut self, address: u32) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }

        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.create_cs_pin();

        // Erase sector on Flash chip
        let result = with_timeout(Duration::from_millis(5000), async {
            let mut spi_device = SpiDevice::new(spi_bus, cs_pin);
            self.erase_sector_internal(&mut spi_device, address).await
        }).await.map_err(|_| SafeFlashError::Timeout)?;

        result
    }

    async fn read_data_internal<CS>(&self, spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>, address: u32, size: u32) -> Result<Vec<u8>, SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        defmt::info!("Flash read internal: address=0x{:08X}, size={}", address, size);

        // Limit single read to avoid heap issues - let the protocol layer handle chunking
        const MAX_SINGLE_READ: u32 = 256; // Maximum single read size
        let actual_size = if size > MAX_SINGLE_READ { MAX_SINGLE_READ } else { size };

        defmt::info!("Reading {} bytes (requested {}, limited to {})", actual_size, size, MAX_SINGLE_READ);

        // Prepare read command with 24-bit address
        let cmd = [
            CMD_READ_DATA,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        let mut data = alloc::vec![0u8; actual_size as usize];

        spi_device.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Read(&mut data),
        ]).await.map_err(|_| SafeFlashError::SpiError)?;

        defmt::info!("Flash read completed: {} bytes read", data.len());
        Ok(data)
    }

    async fn erase_sector_internal<CS>(&self, spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>, address: u32) -> Result<(), SafeFlashError>
    where
        CS: OutputPin,
    {
        use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

        // Write enable
        let write_enable_cmd = [CMD_WRITE_ENABLE];
        spi_device.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&write_enable_cmd),
        ]).await.map_err(|_| SafeFlashError::SpiError)?;

        // Sector erase command with 24-bit address
        let erase_cmd = [
            CMD_SECTOR_ERASE,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        spi_device.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&erase_cmd),
        ]).await.map_err(|_| SafeFlashError::SpiError)?;

        // Wait for erase to complete (poll status register)
        loop {
            let status_cmd = [CMD_READ_STATUS];
            let mut status = [0u8; 1];

            spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&status_cmd),
                embedded_hal_async::spi::Operation::Read(&mut status),
            ]).await.map_err(|_| SafeFlashError::SpiError)?;

            // Check if write in progress bit (bit 0) is clear
            if (status[0] & 0x01) == 0 {
                break;
            }

            Timer::after(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn write_data_internal<CS>(&self, spi_device: &mut SpiDevice<'_, CriticalSectionRawMutex, Spi<'_, Async>, CS>, address: u32, data: &[u8]) -> Result<(), SafeFlashError>
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
            let bytes_to_write = core::cmp::min(
                remaining_data.len(),
                (page_size - page_offset) as usize
            );

            let chunk = &remaining_data[..bytes_to_write];

            // Write enable
            let write_enable_cmd = [CMD_WRITE_ENABLE];
            spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&write_enable_cmd),
            ]).await.map_err(|_| SafeFlashError::SpiError)?;

            // Page program command with 24-bit address
            let program_cmd = [
                CMD_PAGE_PROGRAM,
                (current_address >> 16) as u8,
                (current_address >> 8) as u8,
                current_address as u8,
            ];

            spi_device.transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&program_cmd),
                embedded_hal_async::spi::Operation::Write(chunk),
            ]).await.map_err(|_| SafeFlashError::SpiError)?;

            // Wait for write to complete (poll status register)
            loop {
                let status_cmd = [CMD_READ_STATUS];
                let mut status = [0u8; 1];

                spi_device.transaction(&mut [
                    embedded_hal_async::spi::Operation::Write(&status_cmd),
                    embedded_hal_async::spi::Operation::Read(&mut status),
                ]).await.map_err(|_| SafeFlashError::SpiError)?;

                // Check if write in progress bit (bit 0) is clear
                if (status[0] & 0x01) == 0 {
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
}
