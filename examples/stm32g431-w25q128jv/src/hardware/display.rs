use embassy_stm32::{
    gpio::Output,
    spi::Spi,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use gc9307_async::{Config as DisplayConfig, GC9307C, Orientation, Timer};
use embassy_time;
use crate::resources::{font_renderer_16px::FontRenderer16px, boot_screen_loader::{BootScreenLoader, DisplayTrait}};

// Embassy timer implementation for gc9307-async
struct EmbassyTimer;

impl Timer for EmbassyTimer {
    async fn delay_ms(milliseconds: u64) {
        embassy_time::Timer::after_millis(milliseconds).await;
    }
}

// Display buffer - needs to be static for the lifetime requirement
static mut DISPLAY_BUFFER: [u8; gc9307_async::BUF_SIZE] = [0; gc9307_async::BUF_SIZE];

/// Font character information structure for WenQuanYi bitmap font
#[derive(Debug, Clone)]
struct FontCharInfo {
    unicode: u32,
    width: u8,
    height: u8,
    bitmap_offset: u32,
}

/// Display type alias for easier use
type DisplayType = GC9307C<'static, SpiDevice<'static, CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>, Output<'static>>, Output<'static>, Output<'static>, EmbassyTimer>;

/// Display manager for GC9307 TFT with real hardware driver
pub struct DisplayManager {
    display: Option<DisplayType>,
    width: u16,
    height: u16,
    font_renderer_16px: FontRenderer16px,
    boot_screen_loader: BootScreenLoader,
}

impl DisplayManager {
    /// Create new display manager
    pub fn new() -> Self {
        Self {
            display: None,
            width: 320,  // GC9307 actual resolution for this project
            height: 172,
            font_renderer_16px: FontRenderer16px::new(),
            boot_screen_loader: BootScreenLoader::new(),
        }
    }

    /// Initialize display with real GC9307 driver
    pub async fn initialize(
        &mut self,
        spi_bus: &'static Mutex<CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>>,
        cs_pin: Output<'static>,
        dc_pin: Output<'static>,
        rst_pin: Output<'static>,
    ) -> Result<(), &'static str> {
        defmt::info!("Initializing GC9307 display driver...");

        // Create SPI device
        let spi_device = SpiDevice::new(spi_bus, cs_pin);

        // Configure display (matching reference project)
        let display_config = DisplayConfig {
            rgb: false,
            inverted: false,
            orientation: Orientation::Landscape,
            height: 172,  // Physical height in landscape mode
            width: 320,   // Physical width in landscape mode
            dx: 0,        // No X offset
            dy: 34,       // Y offset as per successful examples
        };

        // Get buffer reference
        let buffer = unsafe { &mut *core::ptr::addr_of_mut!(DISPLAY_BUFFER) };

        // Create display driver
        let mut display = GC9307C::<_, _, _, EmbassyTimer>::new(
            display_config,
            spi_device,
            dc_pin,
            rst_pin,
            buffer,
        );

        // Initialize the display
        display.init().await.map_err(|_| "Failed to initialize GC9307 display")?;

        defmt::info!("✅ GC9307 display initialized successfully");

        // Store the initialized display
        self.display = Some(display);
        self.width = 320;
        self.height = 172;

        Ok(())
    }

    /// Check if display is initialized
    pub fn is_initialized(&self) -> bool {
        self.display.is_some()
    }

    /// Get display dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Clear display with color using real hardware
    pub async fn clear(&mut self, color: Rgb565) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            display.fill_screen(color).await.map_err(|_| "Failed to clear display")?;
            defmt::info!("Display cleared with color");
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Fill rectangle with color (new method from reference project)
    pub async fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Rgb565) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            display.fill_rect(x, y, width, height, color).await.map_err(|_| "Failed to fill rectangle")?;
            defmt::info!("Filled rectangle at ({}, {}) size {}x{}", x, y, width, height);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    // Flash font methods removed - no fonts stored in firmware

    /// Get raw embedded character bitmap (without bit reversal)
    fn get_embedded_char_bitmap_raw(ch: char) -> [u8; 8] {
        match ch {
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            'A' => [0x18, 0x3C, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
            'B' => [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
            'C' => [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
            'D' => [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
            'E' => [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00],
            'F' => [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00],
            'G' => [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00],
            'H' => [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
            'I' => [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
            'J' => [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00],
            'K' => [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
            'L' => [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
            'M' => [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00],
            'N' => [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00],
            'O' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
            'P' => [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
            'Q' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x0E, 0x00],
            'R' => [0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0x00],
            'S' => [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
            'T' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
            'U' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
            'V' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
            'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
            'X' => [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00],
            'Y' => [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
            'Z' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00],
            'a' => [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00],
            'b' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00],
            'c' => [0x00, 0x00, 0x3C, 0x60, 0x60, 0x60, 0x3C, 0x00],
            'd' => [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00],
            'e' => [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00],
            'f' => [0x0E, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x00],
            'g' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x7C],
            'h' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
            'i' => [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00],
            'j' => [0x06, 0x00, 0x0E, 0x06, 0x06, 0x06, 0x66, 0x3C],
            'k' => [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00],
            'l' => [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
            'm' => [0x00, 0x00, 0x66, 0x7F, 0x7F, 0x6B, 0x63, 0x00],
            'n' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
            'o' => [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00],
            'p' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60],
            'q' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06],
            'r' => [0x00, 0x00, 0x7C, 0x66, 0x60, 0x60, 0x60, 0x00],
            's' => [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00],
            't' => [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x0E, 0x00],
            'u' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x00],
            'v' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
            'w' => [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x7F, 0x36, 0x00],
            'x' => [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00],
            'y' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x0C, 0x78],
            'z' => [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00],
            '0' => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
            '1' => [0x18, 0x18, 0x38, 0x18, 0x18, 0x18, 0x7E, 0x00],
            '2' => [0x3C, 0x66, 0x06, 0x0C, 0x30, 0x60, 0x7E, 0x00],
            '3' => [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
            '4' => [0x06, 0x0E, 0x1E, 0x66, 0x7F, 0x06, 0x06, 0x00],
            '5' => [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00],
            '6' => [0x3C, 0x66, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00],
            '7' => [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00],
            '8' => [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00],
            '9' => [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x66, 0x3C, 0x00],
            '!' => [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00],
            ':' => [0x00, 0x18, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00],
            '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
            _ => [0xFF, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0x00],   // Unknown char (box)
        }
    }

    /// Helper function to reverse bits in a byte (LSB->MSB conversion)
    fn reverse_bits(byte: u8) -> u8 {
        let mut result = 0u8;
        for i in 0..8 {
            if (byte & (1 << i)) != 0 {
                result |= 1 << (7 - i);
            }
        }
        result
    }

    /// Get character bitmap from Flash storage using WenQuanYi format
    async fn get_char_bitmap_from_flash(
        ch: char,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(heapless::Vec<u8, 256>, u8, u8), &'static str> {
        let char_code = ch as u32;

        defmt::info!("🔍 NEW FONT FUNCTION: Reading character '{}' (U+{:04X}) from Flash", ch, char_code);

        // First, read the font header to get character count
        let base_address = 0x00020000u32; // Font bitmap address
        let header_data = match flash_manager.read_data_simple(base_address, 4).await {
            Ok(data) => data,
            Err(e) => {
                defmt::error!("Failed to read font header: {}", e);
                return Err("Font header read failed");
            }
        };

        if header_data.len() != 4 {
            return Err("Invalid font header size");
        }

        // Parse character count (little-endian)
        let char_count = u32::from_le_bytes([header_data[0], header_data[1], header_data[2], header_data[3]]);
        defmt::debug!("Font contains {} characters", char_count);

        // Binary search for the character in the character info table
        let char_info_base = base_address + 4; // After 4-byte header
        let char_info = match Self::find_char_info(flash_manager, char_info_base, char_count, char_code).await {
            Ok(info) => info,
            Err(e) => {
                defmt::debug!("Character '{}' (U+{:04X}) not found in font: {}", ch, char_code, e);
                return Err("Character not found in font");
            }
        };

        // Read bitmap data
        // For 12px font: bitmap_offset is now absolute address from font base
        let bitmap_address = base_address + char_info.bitmap_offset;
        let bitmap_size = Self::calculate_bitmap_size(char_info.width, char_info.height);

        // Safety check: ensure bitmap size doesn't exceed read limit
        if bitmap_size > 64 {
            defmt::error!("Bitmap too large: {} bytes (max 64)", bitmap_size);
            return Err("Bitmap too large");
        }

        let bitmap_data = match flash_manager.read_data_simple(bitmap_address, bitmap_size).await {
            Ok(data) => data,
            Err(e) => {
                defmt::error!("Failed to read bitmap data for '{}': {}", ch, e);
                return Err("Bitmap read failed");
            }
        };

        defmt::debug!("Read font bitmap for '{}' ({}x{}, {} bytes) from 0x{:08X}",
                     ch, char_info.width, char_info.height, bitmap_size, bitmap_address);

        // Debug: Print the first few bytes of bitmap data
        if bitmap_data.len() >= 4 {
            defmt::debug!("Bitmap data (first 4 bytes): {:02X} {:02X} {:02X} {:02X}",
                         bitmap_data[0], bitmap_data[1], bitmap_data[2], bitmap_data[3]);
        } else if bitmap_data.len() > 0 {
            defmt::debug!("Bitmap data ({} bytes): first byte = {:02X}", bitmap_data.len(), bitmap_data[0]);
        }

        // Convert to smaller Vec if needed
        let mut result_bitmap = heapless::Vec::<u8, 256>::new();
        for &byte in bitmap_data.iter() {
            result_bitmap.push(byte).map_err(|_| "Bitmap too large")?;
        }

        Ok((result_bitmap, char_info.width, char_info.height))
    }

    /// Binary search for character info in the sorted character table
    /// Updated to use 8-byte format for 12px font: unicode(4) + width(1) + height(1) + bitmap_offset(2)
    async fn find_char_info(flash_manager: &mut crate::hardware::flash::FlashManager, char_info_base: u32, char_count: u32, target_unicode: u32) -> Result<FontCharInfo, &'static str> {
        let mut left = 0u32;
        let mut right = char_count - 1;

        while left <= right {
            let mid = (left + right) / 2;
            let char_info_address = char_info_base + mid * 10; // 10 bytes per character info (correct format)

            // Read character info (10 bytes: 4+1+1+4)
            let char_info_data = match flash_manager.read_data_simple(char_info_address, 10).await {
                Ok(data) => data,
                Err(_) => return Err("Failed to read character info"),
            };

            if char_info_data.len() != 10 {
                return Err("Invalid character info size");
            }

            // Parse character info (10-byte format: Unicode(4) + Width(1) + Height(1) + Offset(4))
            let unicode = u32::from_le_bytes([char_info_data[0], char_info_data[1], char_info_data[2], char_info_data[3]]);
            let width = char_info_data[4];
            let height = char_info_data[5];
            // 32-bit bitmap offset (4 bytes) - correct format
            let bitmap_offset = u32::from_le_bytes([char_info_data[6], char_info_data[7], char_info_data[8], char_info_data[9]]);

            if unicode == target_unicode {
                return Ok(FontCharInfo {
                    unicode,
                    width,
                    height,
                    bitmap_offset,
                });
            } else if unicode < target_unicode {
                left = mid + 1;
            } else {
                if mid == 0 {
                    break;
                }
                right = mid - 1;
            }
        }

        Err("Character not found")
    }

    /// Calculate bitmap size in bytes for given dimensions
    fn calculate_bitmap_size(width: u8, height: u8) -> usize {
        let pixels_per_row = width as usize;
        let bytes_per_row = (pixels_per_row + 7) / 8; // Round up to nearest byte
        bytes_per_row * height as usize
    }

    /// Get bitmap data for a character (8x8 pixels) - embedded fallback
    /// Each byte represents one row of 8 pixels (MSB = leftmost pixel)
    /// Based on standard font8x8 library: https://github.com/dhepper/font8x8
    fn get_char_bitmap_embedded(ch: char) -> [u8; 8] {

        let original = match ch {
            'A' => [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00],
            'B' => [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00],
            'C' => [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00],
            'D' => [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00],
            'E' => [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00],
            'F' => [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00],
            'G' => [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00],
            'H' => [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00],
            'I' => [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
            'J' => [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00],
            'K' => [0x67, 0x66, 0x36, 0x1E, 0x36, 0x66, 0x67, 0x00],
            'L' => [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00],
            'M' => [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00],
            'N' => [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00],
            'O' => [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00],
            'P' => [0x3F, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0F, 0x00],
            'Q' => [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00],
            'R' => [0x3F, 0x66, 0x66, 0x3E, 0x36, 0x66, 0x67, 0x00],
            'S' => [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00],
            'T' => [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
            'U' => [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00],
            'V' => [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00],
            'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
            'X' => [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00],
            'Y' => [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00],
            'Z' => [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00],
            '0' => [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00],
            '1' => [0x0C, 0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x00],
            '2' => [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00],
            '3' => [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00],
            '4' => [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00],
            '5' => [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00],
            '6' => [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00],
            '7' => [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00],
            '8' => [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00],
            '9' => [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00],
            ':' => [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x00],
            '>' => [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00],
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00],
            '!' => [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00],
            _ => [0xFF, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0x00],   // Unknown char (box)
        };

        // Reverse bits in each byte to fix mirrored text issue
        [
            Self::reverse_bits(original[0]),
            Self::reverse_bits(original[1]),
            Self::reverse_bits(original[2]),
            Self::reverse_bits(original[3]),
            Self::reverse_bits(original[4]),
            Self::reverse_bits(original[5]),
            Self::reverse_bits(original[6]),
            Self::reverse_bits(original[7]),
        ]
    }

    // Flash font method removed - no fonts stored in firmware

    /// Verify bitmap data by comparing with known good patterns
    pub async fn verify_flash_bitmap_data(
        &mut self,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        // Get the 'F' character bitmap data
        match Self::get_char_bitmap_from_flash('F', flash_manager).await {
            Ok((bitmap, width, height)) => {
                defmt::info!("🔍 BITMAP VERIFICATION for 'F' ({}x{})", width, height);
                defmt::info!("Raw bitmap data (first 8 bytes): {:?}", &bitmap[..core::cmp::min(8, bitmap.len())]);

                // Expected 'F' pattern (5x7):
                // 11111  -> 0xF8 (MSB) or 0x1F (LSB)
                // 10000  -> 0x80 (MSB) or 0x01 (LSB)
                // 10000  -> 0x80 (MSB) or 0x01 (LSB)
                // 11110  -> 0xF0 (MSB) or 0x0F (LSB)
                // 10000  -> 0x80 (MSB) or 0x01 (LSB)
                // 10000  -> 0x80 (MSB) or 0x01 (LSB)
                // 10000  -> 0x80 (MSB) or 0x01 (LSB)

                // Let's analyze the actual data bit by bit
                for (i, &byte) in bitmap.iter().take(8).enumerate() {
                    defmt::info!("Byte {}: 0x{:02X} = {:08b}", i, byte, byte);
                }

                // Create known good 'F' patterns for comparison
                let known_f_5x7_msb: [u8; 7] = [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0x80]; // MSB first
                let known_f_5x7_lsb: [u8; 7] = [0x1F, 0x01, 0x01, 0x0F, 0x01, 0x01, 0x01]; // LSB first

                defmt::info!("Expected 'F' (MSB): {:?}", known_f_5x7_msb);
                defmt::info!("Expected 'F' (LSB): {:?}", known_f_5x7_lsb);
                defmt::info!("Actual Flash data:   {:?}", &bitmap[..core::cmp::min(7, bitmap.len())]);

                Ok(())
            }
            Err(e) => {
                defmt::error!("Failed to read 'F' for verification: {}", e);
                Err("Failed to read character for verification")
            }
        }
    }

    /// Test different bitmap parsing methods for Flash fonts
    pub async fn test_flash_bitmap_parsing(
        &mut self,
        x: i32,
        y: i32,
        test_char: char,
        color: Rgb565,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            // Read the character bitmap from Flash
            match Self::get_char_bitmap_from_flash(test_char, flash_manager).await {
                Ok((bitmap, width, height)) => {
                    defmt::info!("Testing bitmap parsing for '{}' ({}x{})", test_char, width, height);

                    // Convert to fixed-size array for testing
                    let mut test_bitmap = [0u8; 32];
                    for (i, &byte) in bitmap.iter().enumerate() {
                        if i < 32 {
                            test_bitmap[i] = byte;
                        }
                    }

                    // Test all 4 methods side by side
                    Self::test_bitmap_parsing_methods(display, x, y, &test_bitmap, width, height, color).await?;

                    defmt::info!("Bitmap parsing test complete for '{}'", test_char);
                    Ok(())
                }
                Err(e) => {
                    defmt::error!("Failed to read '{}' for bitmap test: {}", test_char, e);
                    Err("Failed to read character for bitmap test")
                }
            }
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw text at position using WenQuanYi bitmap font from Flash
    pub async fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: Rgb565,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            let mut current_x = x;

            // Define baseline height for vertical alignment
            // Using a common baseline height (e.g., 14px for typical characters)
            const BASELINE_HEIGHT: i32 = 14;

            for ch in text.chars() {
                // MUST use Flash font - no embedded fonts allowed!
                defmt::debug!("Reading character '{}' from Flash", ch);

                // Try to read from Flash using correct font format
                match Self::get_char_bitmap_from_flash(ch, flash_manager).await {
                    Ok((bitmap_vec, width, height)) => {
                        // Calculate vertical offset to align characters to baseline
                        // Characters are aligned so their bottom edge sits on the baseline
                        let y_offset = BASELINE_HEIGHT - height as i32;
                        let char_y = y + y_offset;

                        defmt::debug!("Successfully read '{}' from Flash ({}x{}) at ({}, {}) with y_offset={}", ch, width, height, current_x, char_y, y_offset);
                        // Convert Vec to array for compatibility
                        let mut bitmap_array = [0u8; 32];
                        let copy_len = bitmap_vec.len().min(32);
                        for i in 0..copy_len {
                            bitmap_array[i] = bitmap_vec[i];
                        }
                        Self::draw_char_bitmap_simple_flash(display, current_x, char_y, &bitmap_array, width, height, color).await?;
                        current_x += width as i32 + 1;
                    },
                    Err(e) => {
                        defmt::error!("Failed to read '{}' from Flash: {}", ch, e);
                        // Draw a placeholder rectangle at baseline-aligned position
                        let placeholder_y = y + BASELINE_HEIGHT - 8;
                        display.fill_rect(current_x as u16, placeholder_y as u16, 8, 8, Rgb565::RED).await.map_err(|_| "Failed to draw error placeholder")?;
                        current_x += 9;
                    }
                }
            }

            defmt::info!("Drew text at ({}, {}): '{}'", x, y, text);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw character bitmap with variable dimensions (optimized batch version)
    async fn draw_char_bitmap_inline(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap_data: &[u8],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        // Use write_area for batch rendering instead of pixel-by-pixel
        // This provides massive performance improvement (10-50x faster)
        let bg_color = Rgb565::BLACK; // Transparent pixels use black background

        display.write_area(
            x as u16,
            y as u16,
            width as u16,
            bitmap_data,
            color,
            bg_color
        ).await.map_err(|_| "Failed to draw bitmap with optimized write_area")?;

        defmt::debug!("✅ Drew character bitmap at ({}, {}) size {}x{} using optimized batch rendering", x, y, width, height);
        Ok(())
    }



    /// Draw character bitmap from Flash data (memory-safe version using pixel-by-pixel)
    /// Using MSB first, row-major format (Method 1) - standard font bitmap format
    async fn draw_char_bitmap_simple_flash(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        // Render each pixel of the character using pixel-by-pixel approach
        let bytes_per_row = ((width as usize) + 7) / 8; // Round up to nearest byte

        for row in 0..height {
            for col in 0..width {
                let byte_index = (row as usize) * bytes_per_row + (col as usize) / 8;
                let bit_index = 7 - ((col as usize) % 8); // MSB first - matching web-app exactly

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    let pixel = (byte >> bit_index) & 1; // Extract pixel using shift (web-app style)

                    if pixel != 0 {
                        let pixel_x = x + col as i32;
                        let pixel_y = y + row as i32;

                        // Draw the pixel using fill_rect (1x1 rectangle)
                        display.fill_rect(pixel_x as u16, pixel_y as u16, 1, 1, color)
                            .await.map_err(|_| "Failed to draw pixel")?;
                    }
                }
            }
        }

        defmt::debug!("Drew character bitmap at ({}, {}) size {}x{} using MSB-first pixel-by-pixel", x, y, width, height);
        Ok(())
    }

    /// Test different bitmap parsing methods for Flash fonts
    async fn test_bitmap_parsing_methods(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        let spacing = 40; // Space between different test methods

        // Method 1: MSB first, row-major (original)
        Self::draw_bitmap_method_1(display, x, y, bitmap, width, height, color).await?;

        // Method 2: LSB first, row-major (current)
        Self::draw_bitmap_method_2(display, x + spacing, y, bitmap, width, height, color).await?;

        // Method 3: MSB first, column-major
        Self::draw_bitmap_method_3(display, x + spacing * 2, y, bitmap, width, height, color).await?;

        // Method 4: LSB first, column-major
        Self::draw_bitmap_method_4(display, x + spacing * 3, y, bitmap, width, height, color).await?;

        Ok(())
    }

    /// Method 1: MSB first, row-major (optimized batch approach)
    async fn draw_bitmap_method_1(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        // Use write_area for batch rendering - massive performance improvement
        let bg_color = Rgb565::BLACK; // Transparent pixels use black background

        // Calculate the actual bitmap size needed
        let bytes_per_row = ((width as usize) + 7) / 8;
        let total_bytes = bytes_per_row * (height as usize);
        let bitmap_slice = &bitmap[..total_bytes.min(bitmap.len())];

        display.write_area(
            x as u16,
            y as u16,
            width as u16,
            bitmap_slice,
            color,
            bg_color
        ).await.map_err(|_| "Failed to draw bitmap with optimized write_area")?;

        defmt::debug!("✅ Drew bitmap method 1 at ({}, {}) size {}x{} using optimized batch rendering", x, y, width, height);
        Ok(())
    }

    /// Method 2: LSB first, row-major
    async fn draw_bitmap_method_2(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        let bytes_per_row = ((width as usize) + 7) / 8;

        for row in 0..height {
            let row_start = (row as usize) * bytes_per_row;
            for col in 0..width {
                let byte_index = row_start + (col as usize) / 8;
                let bit_index = (col as usize) % 8; // LSB first

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    if (byte & (1 << bit_index)) != 0 {
                        display.fill_rect((x + col as i32) as u16, (y + row as i32) as u16, 1, 1, color)
                            .await.map_err(|_| "Failed to draw pixel")?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Method 3: MSB first, column-major
    async fn draw_bitmap_method_3(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        let bytes_per_col = ((height as usize) + 7) / 8;

        for col in 0..width {
            let col_start = (col as usize) * bytes_per_col;
            for row in 0..height {
                let byte_index = col_start + (row as usize) / 8;
                let bit_index = 7 - ((row as usize) % 8); // MSB first

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    if (byte & (1 << bit_index)) != 0 {
                        display.fill_rect((x + col as i32) as u16, (y + row as i32) as u16, 1, 1, color)
                            .await.map_err(|_| "Failed to draw pixel")?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Method 4: LSB first, column-major
    async fn draw_bitmap_method_4(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8; 32],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        let bytes_per_col = ((height as usize) + 7) / 8;

        for col in 0..width {
            let col_start = (col as usize) * bytes_per_col;
            for row in 0..height {
                let byte_index = col_start + (row as usize) / 8;
                let bit_index = (row as usize) % 8; // LSB first

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    if (byte & (1 << bit_index)) != 0 {
                        display.fill_rect((x + col as i32) as u16, (y + row as i32) as u16, 1, 1, color)
                            .await.map_err(|_| "Failed to draw pixel")?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Draw filled rectangle using new fill_rect API
    pub async fn draw_rectangle(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Rgb565
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            // Use the new fill_rect method directly
            display.fill_rect(x as u16, y as u16, width as u16, height as u16, color)
                .await.map_err(|_| "Failed to fill rectangle")?;

            defmt::info!("Drew rectangle at ({}, {}) size {}x{}", x, y, width, height);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw text using hardcoded font (H, E, L, O only) for testing
    pub async fn draw_text_hardcoded(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        color: Rgb565
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            let mut current_x = x;

            // Define baseline height for vertical alignment (same as Flash font)
            const BASELINE_HEIGHT: i32 = 14;
            const HARDCODED_CHAR_HEIGHT: i32 = 8;

            for ch in text.chars() {
                let (bitmap, width) = Self::get_hardcoded_char_bitmap(ch);

                // Calculate vertical offset to align characters to baseline
                let y_offset = BASELINE_HEIGHT - HARDCODED_CHAR_HEIGHT;
                let char_y = y + y_offset;

                // Debug: Print hardcoded bitmap data for comparison
                defmt::debug!("Hardcoded '{}' bitmap ({}x8) at ({}, {}) with y_offset={}: {:02X} {:02X} {:02X} {:02X}",
                             ch, width, current_x, char_y, y_offset, bitmap[0], bitmap[1], bitmap[2], bitmap[3]);

                // Draw character using pixel-by-pixel approach
                for row in 0..8 {
                    let byte = bitmap[row];
                    for col in 0..width {
                        if (byte & (1 << (7 - col))) != 0 {
                            let pixel_x = current_x + col as i32;
                            let pixel_y = char_y + row as i32;

                            display.fill_rect(pixel_x as u16, pixel_y as u16, 1, 1, color)
                                .await.map_err(|_| "Failed to draw pixel")?;
                        }
                    }
                }

                current_x += width as i32 + 1; // Add spacing between characters
            }

            defmt::info!("Drew hardcoded text '{}' at ({}, {})", text, x, y);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Get hardcoded bitmap for specific characters (H, E, L, O only)
    fn get_hardcoded_char_bitmap(ch: char) -> ([u8; 8], u8) {
        match ch {
            'H' => ([
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01111110, // ######
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b00000000, //
            ], 6),
            'E' => ([
                0b01111110, // ######
                0b01100000, // ##
                0b01100000, // ##
                0b01111100, // #####
                0b01100000, // ##
                0b01100000, // ##
                0b01111110, // ######
                0b00000000, //
            ], 6),
            'L' => ([
                0b01100000, // ##
                0b01100000, // ##
                0b01100000, // ##
                0b01100000, // ##
                0b01100000, // ##
                0b01100000, // ##
                0b01111110, // ######
                0b00000000, //
            ], 6),
            'O' => ([
                0b00111100, //  ####
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b01100110, // ##  ##
                0b00111100, //  ####
                0b00000000, //
            ], 6),
            _ => ([0; 8], 4), // Unknown character - blank space
        }
    }

    /// Draw pixel (simplified)
    pub async fn draw_pixel(&mut self, x: i32, y: i32, color: Rgb565) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            // Draw a 1x1 rectangle for the pixel using fill_rect
            display.fill_rect(x as u16, y as u16, 1, 1, color)
                .await.map_err(|_| "Failed to draw pixel")?;
            defmt::debug!("Drew pixel at ({}, {})", x, y);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw bitmap using write_area
    pub async fn draw_bitmap(
        &mut self,
        bitmap: &[bool],
        x: i32,
        y: i32,
        width: u8,
        height: u8,
        fg_color: Rgb565,
        bg_color: Option<Rgb565>
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            // Convert bool bitmap to byte bitmap (8 pixels per byte)
            let mut byte_bitmap = [0u8; 128]; // Max 128 bytes for bitmap
            let bytes_needed = (bitmap.len() + 7) / 8;

            for (i, &pixel) in bitmap.iter().enumerate() {
                let byte_index = i / 8;
                let bit_index = 7 - (i % 8);
                if pixel && byte_index < byte_bitmap.len() {
                    byte_bitmap[byte_index] |= 1 << bit_index;
                }
            }

            let bg = bg_color.unwrap_or(Rgb565::BLACK);
            display.write_area(
                x as u16,
                y as u16,
                width as u16,
                &byte_bitmap[..bytes_needed.min(byte_bitmap.len())],
                fg_color,
                bg
            ).await.map_err(|_| "Failed to draw bitmap")?;

            defmt::info!("Drew bitmap at ({}, {}) size {}x{}", x, y, width, height);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw RGB565 image data (simplified)
    pub async fn draw_image(
        &mut self,
        _image_data: &[Rgb565],
        x: i32,
        y: i32,
        width: u16,
        height: u16
    ) -> Result<(), &'static str> {
        if self.display.is_some() {
            defmt::info!("Draw image at ({}, {}) size {}x{}", x, y, width, height);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    // Complex helper function removed - using simplified fill_rect API instead

    /// Draw color bars for testing
    pub async fn draw_color_bars(&mut self) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            defmt::info!("Drawing color bars test pattern");

            const BAR_WIDTH: u16 = 320 / 8; // 8 color bars, 40 pixels each
            const BAR_HEIGHT: u16 = 172;

            let colors = [
                Rgb565::WHITE,
                Rgb565::YELLOW,
                Rgb565::CYAN,
                Rgb565::GREEN,
                Rgb565::MAGENTA,
                Rgb565::RED,
                Rgb565::BLUE,
                Rgb565::BLACK,
            ];

            for (i, &color) in colors.iter().enumerate() {
                let x_start = (i as u16) * BAR_WIDTH;
                let x_end = if i == colors.len() - 1 { 320 } else { x_start + BAR_WIDTH };
                let width = x_end - x_start;

                // Fill the color bar
                display.fill_rect(x_start, 0, width, BAR_HEIGHT, color)
                    .await.map_err(|_| "Failed to fill color bar")?;

                // Small delay to make drawing visible
                embassy_time::Timer::after_millis(100).await;
            }

            defmt::info!("Color bars pattern complete");
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Draw checkerboard pattern for testing (simplified from reference project)
    pub async fn draw_checkerboard(&mut self) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            defmt::info!("Drawing checkerboard test pattern");

            // Clear screen first
            display.fill_screen(Rgb565::BLACK).await.map_err(|_| "Failed to clear screen")?;
            embassy_time::Timer::after_millis(100).await;

            let square_size = 20u16; // 20x20 pixel squares
            let cols = 320 / square_size; // 16 columns
            let rows = 172 / square_size; // 8 rows (with some remainder)

            defmt::info!("Drawing checkerboard: {} cols x {} rows", cols, rows);

            // Draw white squares (black is already the background)
            for row in 0..rows {
                for col in 0..cols {
                    // Only draw white squares in checkerboard pattern
                    let is_white = (row + col) % 2 == 0;
                    if !is_white {
                        continue; // Skip black squares
                    }

                    let x = col * square_size;
                    let y = row * square_size;

                    // Use the new fill_rect method directly
                    display.fill_rect(x, y, square_size, square_size, Rgb565::WHITE)
                        .await.map_err(|_| "Failed to fill square")?;
                }

                // Small delay per row to make drawing visible
                embassy_time::Timer::after_millis(50).await;
            }

            defmt::info!("Checkerboard pattern complete");
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Show startup screen
    pub async fn show_startup_screen(
        &mut self,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        self.clear(Rgb565::BLACK).await?;

        // Draw title
        self.draw_text("Flash Viewer", 60, 30, Rgb565::WHITE, flash_manager).await?;
        self.draw_text("STM32G431", 70, 50, Rgb565::CYAN, flash_manager).await?;

        // Draw border
        self.draw_rectangle(10, 10, 220, 220, Rgb565::BLUE).await?;
        self.draw_rectangle(12, 12, 216, 216, Rgb565::BLACK).await?;

        // Status text
        self.draw_text("Initializing...", 50, 180, Rgb565::YELLOW, flash_manager).await?;

        Ok(())
    }

    /// Show error message
    pub async fn show_error(
        &mut self,
        message: &str,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        self.clear(Rgb565::BLACK).await?;
        self.draw_text("ERROR", 90, 100, Rgb565::RED, flash_manager).await?;
        self.draw_text(message, 20, 120, Rgb565::WHITE, flash_manager).await?;
        Ok(())
    }

    /// Initialize 16px font renderer
    pub async fn initialize_16px_font(&mut self, flash_manager: &mut crate::hardware::flash::FlashManager) -> Result<(), &'static str> {
        defmt::info!("🎨 Initializing 16px font renderer...");
        self.font_renderer_16px.initialize(flash_manager).await?;
        defmt::info!("✅ 16px font renderer initialized successfully");
        Ok(())
    }

    /// Draw text using 16px font
    pub async fn draw_text_16px(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: Rgb565,
        flash_manager: &mut crate::hardware::flash::FlashManager
    ) -> Result<(), &'static str> {
        if let Some(ref mut display) = self.display {
            defmt::info!("🖋️ Drawing 16px text at ({}, {}): '{}'", x, y, text);

            let mut current_x = x;
            const BASELINE_HEIGHT: i32 = 16; // 16px字体的基线高度
            const CHAR_SPACING: i32 = 1;     // 字符间距

            for ch in text.chars() {
                let char_code = ch as u32;

                // 查找字符信息
                match self.font_renderer_16px.find_char(char_code, flash_manager).await {
                    Ok(char_info) => {
                        // 读取字符位图
                        match self.font_renderer_16px.read_char_bitmap(&char_info, flash_manager).await {
                            Ok(bitmap) => {
                                // 计算字符的垂直对齐位置
                                let char_y = y + BASELINE_HEIGHT - char_info.height as i32;

                                // 渲染字符位图
                                Self::render_char_bitmap_16px(
                                    display,
                                    current_x,
                                    char_y,
                                    &bitmap,
                                    char_info.width,
                                    char_info.height,
                                    color
                                ).await?;

                                current_x += char_info.width as i32 + CHAR_SPACING;

                                defmt::debug!("✅ Rendered character '{}' (U+{:04X}) at ({}, {})",
                                             ch, char_code, current_x - char_info.width as i32 - CHAR_SPACING, char_y);
                            },
                            Err(e) => {
                                defmt::error!("❌ Failed to read bitmap for '{}': {}", ch, e);
                                // 绘制占位符
                                display.fill_rect(current_x as u16, y as u16, 8, 16, Rgb565::RED)
                                    .await.map_err(|_| "Failed to draw placeholder")?;
                                current_x += 8 + CHAR_SPACING;
                            }
                        }
                    },
                    Err(e) => {
                        defmt::warn!("⚠️ Character '{}' (U+{:04X}) not found: {}", ch, char_code, e);
                        // 绘制占位符
                        display.fill_rect(current_x as u16, y as u16, 8, 16, Rgb565::YELLOW)
                            .await.map_err(|_| "Failed to draw placeholder")?;
                        current_x += 8 + CHAR_SPACING;
                    }
                }
            }

            defmt::info!("✅ 16px text rendered successfully: '{}'", text);
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Render character bitmap for 16px font
    async fn render_char_bitmap_16px(
        display: &mut DisplayType,
        x: i32,
        y: i32,
        bitmap: &[u8],
        width: u8,
        height: u8,
        color: Rgb565
    ) -> Result<(), &'static str> {
        let bytes_per_row = ((width as usize) + 7) / 8;

        for row in 0..height {
            for col in 0..width {
                let byte_index = (row as usize) * bytes_per_row + (col as usize) / 8;
                let bit_index = 7 - ((col as usize) % 8); // MSB优先

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    let pixel = (byte >> bit_index) & 1;

                    if pixel != 0 {
                        let pixel_x = x + col as i32;
                        let pixel_y = y + row as i32;

                        // 绘制像素
                        display.fill_rect(pixel_x as u16, pixel_y as u16, 1, 1, color)
                            .await.map_err(|_| "Failed to draw pixel")?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Show boot screen
    pub async fn show_boot_screen(&mut self, flash_manager: &mut crate::hardware::flash::FlashManager) -> Result<(), &'static str> {
        defmt::info!("🔍 DEBUG: Entered show_boot_screen method");
        defmt::info!("🖼️ Loading and displaying boot screen...");

        defmt::info!("🔍 DEBUG: About to call verify_screen_data");
        // 验证开屏图数据
        self.boot_screen_loader.verify_screen_data(flash_manager).await?;
        defmt::info!("🔍 DEBUG: verify_screen_data completed successfully");

        // 获取开屏图信息
        let screen_info = self.boot_screen_loader.get_screen_info();
        defmt::info!("📊 Boot screen info: {}x{} pixels, {} bytes",
                    screen_info.width, screen_info.height, screen_info.total_size);

        if let Some(ref mut display) = self.display {
            // 清空屏幕
            display.fill_screen(Rgb565::BLACK).await.map_err(|_| "Failed to clear screen")?;

            // 加载并显示开屏图
            self.boot_screen_loader.load_and_display(display, flash_manager).await?;

            defmt::info!("✅ Boot screen displayed successfully!");
            Ok(())
        } else {
            Err("Display not initialized")
        }
    }

    /// Get boot screen statistics
    pub async fn get_boot_screen_stats(&mut self, flash_manager: &mut crate::hardware::flash::FlashManager) -> Result<(), &'static str> {
        match self.boot_screen_loader.get_screen_stats(flash_manager).await {
            Ok(stats) => {
                defmt::info!("📊 Boot screen statistics:");
                defmt::info!("   Size: {}x{} pixels ({} bytes)", stats.width, stats.height, stats.total_size);
                defmt::info!("   Sampled: {} pixels", stats.sampled_pixels);
                defmt::info!("   Average RGB: ({}, {}, {})", stats.avg_red, stats.avg_green, stats.avg_blue);
                Ok(())
            },
            Err(e) => {
                defmt::error!("❌ Failed to get boot screen stats: {}", e);
                Err(e)
            }
        }
    }
}

/// Implement DisplayTrait for our DisplayType to enable boot screen loading
impl DisplayTrait for DisplayType {
    type Error = &'static str;

    async fn fill_screen(&mut self, color: Rgb565) -> Result<(), Self::Error> {
        self.fill_screen(color).await.map_err(|_| "Failed to fill screen")
    }

    async fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Rgb565) -> Result<(), Self::Error> {
        self.fill_rect(x, y, width, height, color).await.map_err(|_| "Failed to fill rect")
    }

    /// Draw single pixel (original method)
    async fn draw_pixel(&mut self, x: u16, y: u16, color: Rgb565) -> Result<(), Self::Error> {
        self.fill_rect(x, y, 1, 1, color).await.map_err(|_| "Failed to draw pixel")
    }
}
