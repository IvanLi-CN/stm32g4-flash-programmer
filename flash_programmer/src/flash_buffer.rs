use defmt::*;
use core::ptr;

/// Flash programming buffer protocol
/// 
/// Memory layout:
/// - Bytes 0-3: Magic number (0xDEADBEEF)
/// - Bytes 4-7: Target flash address (little-endian u32)
/// - Bytes 8-11: Data length (little-endian u32)
/// - Bytes 12-15: Status (little-endian u32)
/// - Bytes 16-2047: Data payload (2032 bytes)

const MAGIC_NUMBER: u32 = 0xDEADBEEF;
const BUFFER_SIZE: usize = 2048;
const HEADER_SIZE: usize = 16;
const DATA_SIZE: usize = BUFFER_SIZE - HEADER_SIZE;

/// Buffer status values
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum BufferStatus {
    Idle = 0,           // Buffer is empty, ready for new data
    HasData = 1,        // Buffer contains data to be programmed
    Programming = 2,    // STM32 is currently programming the data
    Complete = 3,       // Programming completed successfully
    Error = 4,          // Programming failed
    VerifyRequest = 5,  // Request to verify flash data
    VerifyComplete = 6, // Verification completed successfully
    VerifyError = 7,    // Verification failed
}

impl From<u32> for BufferStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => BufferStatus::Idle,
            1 => BufferStatus::HasData,
            2 => BufferStatus::Programming,
            3 => BufferStatus::Complete,
            4 => BufferStatus::Error,
            5 => BufferStatus::VerifyRequest,
            6 => BufferStatus::VerifyComplete,
            7 => BufferStatus::VerifyError,
            _ => BufferStatus::Error,
        }
    }
}

/// Flash programming buffer interface
pub struct FlashBuffer {
    buffer_ptr: *mut u8,
}

impl FlashBuffer {
    /// Create a new FlashBuffer instance
    /// 
    /// # Safety
    /// This function is unsafe because it accesses memory at a fixed address
    /// defined by the linker script. The caller must ensure that the memory
    /// layout is correct and the buffer is properly allocated.
    pub unsafe fn new() -> Self {
        extern "C" {
            static _flash_buffer_start: u8;
        }
        
        let buffer_ptr = ptr::addr_of!(_flash_buffer_start) as *mut u8;
        
        info!("Flash buffer initialized at address: 0x{:08X}", buffer_ptr as u32);
        
        Self { buffer_ptr }
    }
    
    /// Read the magic number from the buffer
    fn read_magic(&self) -> u32 {
        unsafe {
            ptr::read_volatile(self.buffer_ptr as *const u32)
        }
    }
    
    /// Read the target flash address
    pub fn read_address(&self) -> u32 {
        unsafe {
            ptr::read_volatile(self.buffer_ptr.add(4) as *const u32)
        }
    }

    /// Read the data length
    pub fn read_length(&self) -> u32 {
        unsafe {
            ptr::read_volatile(self.buffer_ptr.add(8) as *const u32)
        }
    }
    
    /// Read the current status
    pub fn read_status(&self) -> BufferStatus {
        let status_value = unsafe {
            ptr::read_volatile(self.buffer_ptr.add(12) as *const u32)
        };
        BufferStatus::from(status_value)
    }
    
    /// Write the status
    pub fn write_status(&self, status: BufferStatus) {
        unsafe {
            ptr::write_volatile(self.buffer_ptr.add(12) as *mut u32, status as u32);
        }
    }
    
    /// Check if the buffer contains valid data
    pub fn has_valid_data(&self) -> bool {
        let magic = self.read_magic();
        let status = self.read_status();
        let length = self.read_length();

        magic == MAGIC_NUMBER &&
        (status == BufferStatus::HasData || status == BufferStatus::Programming) &&
        length > 0 &&
        length <= DATA_SIZE as u32
    }

    /// Check if the buffer contains a verify request
    pub fn has_verify_request(&self) -> bool {
        let magic = self.read_magic();
        let status = self.read_status();

        magic == 0xCAFEBABE &&  // Verify command magic
        status == BufferStatus::VerifyRequest
    }
    
    /// Get the programming request details
    pub fn get_request(&self) -> Option<ProgrammingRequest> {
        if !self.has_valid_data() {
            return None;
        }
        
        let address = self.read_address();
        let length = self.read_length();
        
        Some(ProgrammingRequest {
            address,
            length: length as usize,
        })
    }
    
    /// Read data from the buffer
    pub fn read_data(&self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.has_valid_data() {
            return Err("No valid data in buffer");
        }
        
        let length = self.read_length() as usize;
        if buffer.len() < length {
            return Err("Output buffer too small");
        }
        
        if length > DATA_SIZE {
            return Err("Data length exceeds buffer capacity");
        }
        
        unsafe {
            let data_ptr = self.buffer_ptr.add(HEADER_SIZE);
            ptr::copy_nonoverlapping(data_ptr, buffer.as_mut_ptr(), length);
        }
        
        Ok(length)
    }
    
    /// Clear the buffer (set status to Idle and clear magic)
    pub fn clear(&self) {
        unsafe {
            // Clear magic number
            ptr::write_volatile(self.buffer_ptr as *mut u32, 0);
            // Set status to Idle
            ptr::write_volatile(self.buffer_ptr.add(12) as *mut u32, BufferStatus::Idle as u32);
        }
    }
    
    /// Get buffer statistics for debugging
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            magic: self.read_magic(),
            address: self.read_address(),
            length: self.read_length(),
            status: self.read_status(),
        }
    }
}

/// Programming request details
#[derive(Debug)]
pub struct ProgrammingRequest {
    pub address: u32,
    pub length: usize,
}

/// Buffer statistics for debugging
#[derive(Debug)]
pub struct BufferStats {
    pub magic: u32,
    pub address: u32,
    pub length: u32,
    pub status: BufferStatus,
}

impl defmt::Format for BufferStats {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "BufferStats {{ magic: 0x{:08X}, address: 0x{:08X}, length: {}, status: {:?} }}",
            self.magic,
            self.address,
            self.length,
            self.status
        );
    }
}

impl defmt::Format for ProgrammingRequest {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ProgrammingRequest {{ address: 0x{:08X}, length: {} }}",
            self.address,
            self.length
        );
    }
}

impl defmt::Format for BufferStatus {
    fn format(&self, fmt: defmt::Formatter) {
        let status_str = match self {
            BufferStatus::Idle => "Idle",
            BufferStatus::HasData => "HasData",
            BufferStatus::Programming => "Programming",
            BufferStatus::Complete => "Complete",
            BufferStatus::Error => "Error",
            BufferStatus::VerifyRequest => "VerifyRequest",
            BufferStatus::VerifyComplete => "VerifyComplete",
            BufferStatus::VerifyError => "VerifyError",
        };
        defmt::write!(fmt, "{}", status_str);
    }
}
