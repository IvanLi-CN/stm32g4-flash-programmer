use heapless::Vec;

/// Font bitmap header structure
#[derive(Debug, Clone)]
pub struct FontHeader {
    pub char_count: u32,
}

/// Character information structure
#[derive(Debug, Clone)]
pub struct CharInfo {
    pub unicode: u32,
    pub width: u8,
    pub height: u8,
    pub bitmap_offset: u16,
}

/// Font bitmap parser for the custom format
pub struct FontParser;

impl FontParser {
    /// Parse font header from raw data
    pub fn parse_header(data: &[u8]) -> Result<FontHeader, &'static str> {
        if data.len() < 4 {
            return Err("Insufficient data for header");
        }
        
        let char_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        Ok(FontHeader { char_count })
    }
    
    /// Parse character information table
    pub fn parse_char_info(data: &[u8], char_index: usize) -> Result<CharInfo, &'static str> {
        let offset = 4 + char_index * 8; // Header (4 bytes) + char_index * 8 bytes per char
        
        if data.len() < offset + 8 {
            return Err("Insufficient data for character info");
        }
        
        let unicode = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]);
        let width = data[offset + 4];
        let height = data[offset + 5];
        let bitmap_offset = u16::from_le_bytes([data[offset + 6], data[offset + 7]]);
        
        Ok(CharInfo {
            unicode,
            width,
            height,
            bitmap_offset,
        })
    }
    
    /// Find character by Unicode code point
    pub fn find_char_by_unicode(
        data: &[u8], 
        unicode: u32
    ) -> Result<Option<(usize, CharInfo)>, &'static str> {
        let header = Self::parse_header(data)?;
        
        for i in 0..header.char_count as usize {
            let char_info = Self::parse_char_info(data, i)?;
            if char_info.unicode == unicode {
                return Ok(Some((i, char_info)));
            }
        }
        
        Ok(None)
    }
    
    /// Extract bitmap data for a character
    pub fn extract_bitmap(
        data: &[u8],
        char_info: &CharInfo,
        header: &FontHeader
    ) -> Result<Vec<u8, 256>, &'static str> {
        // Calculate bitmap data start position
        let bitmap_start = 4 + (header.char_count as usize * 8) + char_info.bitmap_offset as usize;
        
        // Calculate bitmap size in bytes (1 bit per pixel, rounded up to bytes)
        let bitmap_size_bits = char_info.width as usize * char_info.height as usize;
        let bitmap_size_bytes = (bitmap_size_bits + 7) / 8;
        
        if data.len() < bitmap_start + bitmap_size_bytes {
            return Err("Insufficient data for bitmap");
        }
        
        let mut bitmap = Vec::new();
        for i in 0..bitmap_size_bytes {
            bitmap.push(data[bitmap_start + i]).map_err(|_| "Bitmap buffer full")?;
        }
        
        Ok(bitmap)
    }
    
    /// Convert 1-bit bitmap to pixel array (for display)
    pub fn bitmap_to_pixels(
        bitmap: &[u8],
        width: u8,
        height: u8
    ) -> Result<Vec<bool, 256>, &'static str> {
        let mut pixels = Vec::new();
        
        for y in 0..height {
            for x in 0..width {
                let bit_index = y as usize * width as usize + x as usize;
                let byte_index = bit_index / 8;
                let bit_offset = bit_index % 8;
                
                if byte_index >= bitmap.len() {
                    return Err("Bitmap data insufficient");
                }
                
                let pixel = (bitmap[byte_index] >> (7 - bit_offset)) & 1 != 0;
                pixels.push(pixel).map_err(|_| "Pixel buffer full")?;
            }
        }
        
        Ok(pixels)
    }
}

/// Font character ranges (from README)
pub mod char_ranges {
    /// ASCII printable characters
    pub const ASCII_START: u32 = 0x0021; // !
    pub const ASCII_END: u32 = 0x007E;   // ~
    pub const ASCII_COUNT: u32 = 94;
    
    /// CJK Unified Ideographs
    pub const CJK_START: u32 = 0x4E00;
    pub const CJK_END: u32 = 0x51E7;
    pub const CJK_COUNT: u32 = 1000;
    
    /// CJK Extension A
    pub const CJK_EXT_A_START: u32 = 0x3400;
    pub const CJK_EXT_A_END: u32 = 0x37E7;
    pub const CJK_EXT_A_COUNT: u32 = 1000;
    
    /// Total character count
    pub const TOTAL_CHARS: u32 = ASCII_COUNT + CJK_COUNT + CJK_EXT_A_COUNT; // 2094
    
    /// Check if Unicode is in supported range
    pub fn is_supported_unicode(unicode: u32) -> bool {
        (unicode >= ASCII_START && unicode <= ASCII_END) ||
        (unicode >= CJK_START && unicode <= CJK_END) ||
        (unicode >= CJK_EXT_A_START && unicode <= CJK_EXT_A_END)
    }
    
    /// Get character range name
    pub fn get_range_name(unicode: u32) -> &'static str {
        if unicode >= ASCII_START && unicode <= ASCII_END {
            "ASCII"
        } else if unicode >= CJK_START && unicode <= CJK_END {
            "CJK Unified"
        } else if unicode >= CJK_EXT_A_START && unicode <= CJK_EXT_A_END {
            "CJK Extension A"
        } else {
            "Unsupported"
        }
    }
}
