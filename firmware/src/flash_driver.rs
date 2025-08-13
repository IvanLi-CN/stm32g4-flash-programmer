use embassy_time::{Duration, Timer};
use embedded_hal_async::spi::SpiDevice;
use flash_protocol::*;

// W25Q128 Commands
const CMD_READ_JEDEC_ID: u8 = 0x9F;
const CMD_READ_STATUS: u8 = 0x05;
const CMD_WRITE_ENABLE: u8 = 0x06;
const CMD_WRITE_DISABLE: u8 = 0x04;
const CMD_READ_DATA: u8 = 0x03;
const CMD_PAGE_PROGRAM: u8 = 0x02;
const CMD_SECTOR_ERASE: u8 = 0x20;
const CMD_CHIP_ERASE: u8 = 0xC7;

// Status register bits
const STATUS_BUSY: u8 = 0x01;
const STATUS_WEL: u8 = 0x02;

#[derive(Debug)]
pub enum FlashDriverError {
    SpiError,
    InvalidAddress,
    InvalidSize,
    NotInitialized,
    Timeout,
    WriteNotEnabled,
}

pub struct FlashDriver<SPI> {
    spi: SPI,
    initialized: bool,
}

impl<SPI> FlashDriver<SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi_device: SPI) -> Self {
        Self {
            spi: spi_device,
            initialized: false,
        }
    }

    pub async fn init(&mut self) -> Result<(), FlashDriverError> {
        // Read JEDEC ID to verify connection
        let jedec_id = self.read_jedec_id().await?;
        // Flash JEDEC ID logged (removed defmt dependency)

        // Verify it's a W25Q128 (0xEF4018)
        if jedec_id != 0xEF4018 {
            // Unexpected JEDEC ID (removed defmt dependency)
        }

        self.initialized = true;
        // Flash driver initialized successfully (removed defmt dependency)
        Ok(())
    }

    async fn read_jedec_id(&mut self) -> Result<u32, FlashDriverError> {
        let mut cmd = [CMD_READ_JEDEC_ID];
        let mut response = [0u8; 3];

        self.spi.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Read(&mut response),
        ]).await.map_err(|_| FlashDriverError::SpiError)?;

        let jedec_id = ((response[0] as u32) << 16) |
                      ((response[1] as u32) << 8) |
                      (response[2] as u32);

        Ok(jedec_id)
    }

    async fn read_status(&mut self) -> Result<u8, FlashDriverError> {
        let mut cmd = [CMD_READ_STATUS];
        let mut status = [0u8; 1];

        self.spi.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Read(&mut status),
        ]).await.map_err(|_| FlashDriverError::SpiError)?;

        Ok(status[0])
    }

    async fn write_enable(&mut self) -> Result<(), FlashDriverError> {
        let cmd = [CMD_WRITE_ENABLE];

        self.spi.write(&cmd).await.map_err(|_| FlashDriverError::SpiError)?;

        // Verify write enable was successful
        let status = self.read_status().await?;
        if (status & STATUS_WEL) == 0 {
            return Err(FlashDriverError::WriteNotEnabled);
        }

        Ok(())
    }

    async fn wait_for_ready(&mut self) -> Result<(), FlashDriverError> {
        let mut timeout = 1000; // 1000 * 10ms = 10 second timeout

        while timeout > 0 {
            let status = self.read_status().await?;
            if (status & STATUS_BUSY) == 0 {
                // Not busy
                return Ok(());
            }

            Timer::after(Duration::from_millis(10)).await;
            timeout -= 1;
        }

        // Flash operation timeout (removed defmt dependency)
        Err(FlashDriverError::Timeout)
    }

    pub async fn get_info(&mut self) -> Result<FlashInfo, FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        let jedec_id = self.read_jedec_id().await?;

        Ok(FlashInfo {
            jedec_id,
            total_size: FLASH_TOTAL_SIZE as u32,
            page_size: FLASH_PAGE_SIZE as u32,
            sector_size: FLASH_SECTOR_SIZE as u32,
        })
    }

    pub async fn erase_sector(&mut self, address: u32) -> Result<(), FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        // Validate address alignment
        if address % FLASH_SECTOR_SIZE as u32 != 0 {
            return Err(FlashDriverError::InvalidAddress);
        }

        if address >= FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidAddress);
        }

        // Erasing sector (removed defmt dependency)

        // Enable write
        self.write_enable().await?;

        // Send erase command
        let cmd = [
            CMD_SECTOR_ERASE,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.spi.write(&cmd).await.map_err(|_| FlashDriverError::SpiError)?;

        // Wait for erase to complete
        self.wait_for_ready().await?;

        // Sector erase completed (removed defmt dependency)
        Ok(())
    }

    pub async fn erase_range(&mut self, start_address: u32, size: u32) -> Result<(), FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        if start_address >= FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidAddress);
        }

        if start_address + size > FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidSize);
        }

        // Calculate sector boundaries
        let start_sector = (start_address / FLASH_SECTOR_SIZE as u32) * FLASH_SECTOR_SIZE as u32;
        let end_address = start_address + size;
        let end_sector = ((end_address + FLASH_SECTOR_SIZE as u32 - 1) / FLASH_SECTOR_SIZE as u32) * FLASH_SECTOR_SIZE as u32;

        // Erasing range (removed defmt dependency)

        let mut current_sector = start_sector;
        while current_sector < end_sector {
            self.erase_sector(current_sector).await?;
            current_sector += FLASH_SECTOR_SIZE as u32;
        }

        Ok(())
    }

    pub async fn write_page(&mut self, address: u32, data: &[u8]) -> Result<(), FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        if data.len() > FLASH_PAGE_SIZE {
            return Err(FlashDriverError::InvalidSize);
        }

        if address >= FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidAddress);
        }

        // Check page boundary
        let page_start = (address / FLASH_PAGE_SIZE as u32) * FLASH_PAGE_SIZE as u32;
        if address + data.len() as u32 > page_start + FLASH_PAGE_SIZE as u32 {
            return Err(FlashDriverError::InvalidSize);
        }

        // Enable write
        self.write_enable().await?;

        // Prepare command with address
        let mut cmd = [0u8; 4];
        cmd[0] = CMD_PAGE_PROGRAM;
        cmd[1] = (address >> 16) as u8;
        cmd[2] = (address >> 8) as u8;
        cmd[3] = address as u8;

        // Write command + address + data
        self.spi.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Write(data),
        ]).await.map_err(|_| FlashDriverError::SpiError)?;

        // Wait for write to complete
        self.wait_for_ready().await?;
        Ok(())
    }

    pub async fn write_data(&mut self, mut address: u32, data: &[u8]) -> Result<(), FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        if address >= FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidAddress);
        }

        if address + data.len() as u32 > FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidSize);
        }

        let mut remaining = data;
        
        while !remaining.is_empty() {
            // Calculate how much we can write in this page
            let page_start = (address / FLASH_PAGE_SIZE as u32) * FLASH_PAGE_SIZE as u32;
            let page_offset = address - page_start;
            let page_remaining = FLASH_PAGE_SIZE as u32 - page_offset;
            
            let write_size = core::cmp::min(remaining.len() as u32, page_remaining) as usize;
            
            // Write this chunk
            self.write_page(address, &remaining[..write_size]).await?;
            
            // Move to next chunk
            address += write_size as u32;
            remaining = &remaining[write_size..];
        }

        Ok(())
    }

    pub async fn read_data(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), FlashDriverError> {
        if !self.initialized {
            return Err(FlashDriverError::NotInitialized);
        }

        if address >= FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidAddress);
        }

        if address + buffer.len() as u32 > FLASH_TOTAL_SIZE as u32 {
            return Err(FlashDriverError::InvalidSize);
        }

        // Prepare read command with address
        let cmd = [
            CMD_READ_DATA,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        // Read data
        self.spi.transaction(&mut [
            embedded_hal_async::spi::Operation::Write(&cmd),
            embedded_hal_async::spi::Operation::Read(buffer),
        ]).await.map_err(|_| FlashDriverError::SpiError)?;

        Ok(())
    }


}

#[derive(Debug)]
pub struct FlashInfo {
    pub jedec_id: u32,
    pub total_size: u32,
    pub page_size: u32,
    pub sector_size: u32,
}
