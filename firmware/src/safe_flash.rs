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
    cs_pin: Option<Output<'static>>,
    initialized: bool,
    flash_available: bool,
}

impl SafeFlashManager {
    pub fn new() -> Self {
        Self {
            spi_bus: None,
            cs_pin: None,
            initialized: false,
            flash_available: false,
        }
    }
    
    pub fn set_spi_resources(
        &mut self, 
        spi_bus: &'static Mutex<CriticalSectionRawMutex, Spi<'static, Async>>,
        cs_pin: Output<'static>
    ) {
        self.spi_bus = Some(spi_bus);
        self.cs_pin = Some(cs_pin);
    }
    
    pub async fn try_initialize(&mut self) -> Result<(), SafeFlashError> {
        if self.initialized {
            return if self.flash_available { Ok(()) } else { Err(SafeFlashError::InitializationFailed) };
        }
        
        let spi_bus = self.spi_bus.ok_or(SafeFlashError::NotInitialized)?;
        let cs_pin = self.cs_pin.take().ok_or(SafeFlashError::NotInitialized)?;
        
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
        // For now, always return mock data to avoid blocking
        // TODO: Implement real SPI Flash detection when hardware is connected
        let flash_info = FlashInfo {
            jedec_id: 0xEF4018, // Mock W25Q128 JEDEC ID
            total_size: 16 * 1024 * 1024, // 16MB
            page_size: 256,
            sector_size: 4096,
        };

        Ok(flash_info)
    }
    
    pub fn is_available(&self) -> bool {
        self.initialized && self.flash_available
    }
    
    pub async fn read_data(&mut self, _address: u32, size: u32) -> Result<Vec<u8>, SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }
        
        // For now, return mock data
        // TODO: Implement real SPI Flash read
        let data = alloc::vec![0xAA; size as usize];
        Ok(data)
    }
    
    pub async fn write_data(&mut self, _address: u32, _data: &[u8]) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }
        
        // For now, just simulate success
        // TODO: Implement real SPI Flash write
        Timer::after(Duration::from_millis(10)).await; // Simulate write time
        Ok(())
    }
    
    pub async fn erase_sector(&mut self, _address: u32) -> Result<(), SafeFlashError> {
        if !self.is_available() {
            return Err(SafeFlashError::NotInitialized);
        }
        
        // For now, just simulate success
        // TODO: Implement real SPI Flash erase
        Timer::after(Duration::from_millis(50)).await; // Simulate erase time
        Ok(())
    }
}
