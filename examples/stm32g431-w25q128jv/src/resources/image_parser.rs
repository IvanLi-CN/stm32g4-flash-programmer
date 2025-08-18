use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::RgbColor,
};

/// RGB565 image information
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u16,
    pub height: u16,
    pub format: ImageFormat,
}

/// Supported image formats
#[derive(Debug, Clone)]
pub enum ImageFormat {
    Rgb565,
}

/// Image parser for RGB565 format
pub struct ImageParser;

impl ImageParser {
    /// Parse boot screen image (320x172 RGB565)
    pub fn parse_boot_screen_info() -> ImageInfo {
        ImageInfo {
            width: 320,
            height: 172,
            format: ImageFormat::Rgb565,
        }
    }

    /// Convert raw RGB565 data to pixel color
    pub fn rgb565_to_color(data: &[u8], pixel_index: usize) -> Result<Rgb565, &'static str> {
        let byte_index = pixel_index * 2;

        if data.len() < byte_index + 2 {
            return Err("Insufficient data for pixel");
        }

        // RGB565 is stored in little-endian format
        let rgb565_value = u16::from_le_bytes([data[byte_index], data[byte_index + 1]]);

        Ok(Rgb565::new(
            ((rgb565_value >> 11) & 0x1F) as u8,  // Red (5 bits)
            ((rgb565_value >> 5) & 0x3F) as u8,   // Green (6 bits)
            (rgb565_value & 0x1F) as u8,          // Blue (5 bits)
        ))
    }

    /// Get pixel color at specific coordinates
    pub fn get_pixel_at(
        data: &[u8],
        info: &ImageInfo,
        x: u16,
        y: u16
    ) -> Result<Rgb565, &'static str> {
        if x >= info.width || y >= info.height {
            return Err("Coordinates out of bounds");
        }

        let pixel_index = (y as usize * info.width as usize + x as usize);
        Self::rgb565_to_color(data, pixel_index)
    }

    /// Extract a rectangular region from the image
    pub fn extract_region(
        data: &[u8],
        info: &ImageInfo,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16
    ) -> Result<heapless::Vec<Rgb565, 1024>, &'static str> {
        if start_x + width > info.width || start_y + height > info.height {
            return Err("Region out of bounds");
        }

        let mut pixels = heapless::Vec::new();

        for y in start_y..(start_y + height) {
            for x in start_x..(start_x + width) {
                let color = Self::get_pixel_at(data, info, x, y)?;
                pixels.push(color).map_err(|_| "Pixel buffer full")?;
            }
        }

        Ok(pixels)
    }

    /// Calculate image statistics
    pub fn calculate_stats(data: &[u8], info: &ImageInfo) -> ImageStats {
        let total_pixels = info.width as u32 * info.height as u32;
        let expected_size = total_pixels * 2; // 2 bytes per RGB565 pixel

        let mut red_sum = 0u32;
        let mut green_sum = 0u32;
        let mut blue_sum = 0u32;
        let mut valid_pixels = 0u32;

        // Sample every 16th pixel for performance
        for i in (0..total_pixels).step_by(16) {
            if let Ok(color) = Self::rgb565_to_color(data, i as usize) {
                red_sum += color.r() as u32;
                green_sum += color.g() as u32;
                blue_sum += color.b() as u32;
                valid_pixels += 1;
            }
        }

        ImageStats {
            width: info.width,
            height: info.height,
            total_pixels,
            data_size: data.len() as u32,
            expected_size,
            avg_red: if valid_pixels > 0 { red_sum / valid_pixels } else { 0 },
            avg_green: if valid_pixels > 0 { green_sum / valid_pixels } else { 0 },
            avg_blue: if valid_pixels > 0 { blue_sum / valid_pixels } else { 0 },
            sampled_pixels: valid_pixels,
        }
    }

    /// Generate ASCII art preview of the image
    pub fn generate_ascii_preview(
        data: &[u8],
        info: &ImageInfo,
        preview_width: u16,
        preview_height: u16
    ) -> Result<heapless::String<1024>, &'static str> {
        let mut preview = heapless::String::new();

        let x_step = info.width / preview_width;
        let y_step = info.height / preview_height;

        for y in 0..preview_height {
            for x in 0..preview_width {
                let sample_x = x * x_step;
                let sample_y = y * y_step;

                if let Ok(color) = Self::get_pixel_at(data, info, sample_x, sample_y) {
                    // Convert to grayscale and map to ASCII characters
                    let gray = (color.r() as u16 + color.g() as u16 + color.b() as u16) / 3;
                    let ascii_char = match gray {
                        0..=7 => ' ',
                        8..=15 => '.',
                        16..=23 => ':',
                        24..=31 => '-',
                        _ => '#',
                    };
                    preview.push(ascii_char).map_err(|_| "Preview buffer full")?;
                } else {
                    preview.push('?').map_err(|_| "Preview buffer full")?;
                }
            }
            preview.push('\n').map_err(|_| "Preview buffer full")?;
        }

        Ok(preview)
    }
}

/// Image statistics
#[derive(Debug)]
pub struct ImageStats {
    pub width: u16,
    pub height: u16,
    pub total_pixels: u32,
    pub data_size: u32,
    pub expected_size: u32,
    pub avg_red: u32,
    pub avg_green: u32,
    pub avg_blue: u32,
    pub sampled_pixels: u32,
}
