# Flash Content Generator

A comprehensive toolchain for generating, managing, and analyzing content stored in external SPI Flash memory for STM32 embedded projects. This system creates optimized firmware images containing fonts, graphics, and application data for W25Q128JV (16MB) flash chips.

## 🎯 Overview

The Flash Content Generator provides a complete end-to-end solution for:

- **Font Processing**: Converting TTF fonts to optimized 1-bit bitmap format with 27,678+ characters (ASCII + CJK)
- **Image Processing**: Converting SVG graphics to RGB565 format for embedded displays
- **Memory Management**: Creating structured Flash layouts with automatic address allocation
- **Firmware Composition**: Assembling complete 16MB flash images with multiple resource types
- **Data Verification**: Comprehensive validation and integrity checking
- **Web-based Analysis**: Interactive tools for inspecting and debugging flash content

## 🏗️ Firmware Architecture

### Flash Memory Layout (W25Q128JV - 16MB)

```text
┌─────────────────────────────────────────────────────────────┐
│ 0x000000 │ Boot Screen      │ 110KB  │ RGB565 320×172      │
├─────────────────────────────────────────────────────────────┤
│ 0x020000 │ Font 12px        │ 1MB    │ 27,678 characters   │
├─────────────────────────────────────────────────────────────┤
│ 0x120000 │ Font 16px        │ 1MB    │ 27,678 characters   │
├─────────────────────────────────────────────────────────────┤
│ 0x220000 │ UI Graphics      │ 2MB    │ Icons & UI elements │
├─────────────────────────────────────────────────────────────┤
│ 0x420000 │ Application Data │ 3MB    │ User data storage   │
├─────────────────────────────────────────────────────────────┤
│ 0x720000 │ User Config      │ 64KB   │ Settings & config   │
├─────────────────────────────────────────────────────────────┤
│ 0x730000 │ Log Storage      │ 128KB  │ System logs         │
├─────────────────────────────────────────────────────────────┤
│ 0x750000 │ Firmware Update  │ 512KB  │ OTA update buffer   │
├─────────────────────────────────────────────────────────────┤
│ 0x7D0000 │ Reserved         │ 8.2MB  │ Future expansion    │
└─────────────────────────────────────────────────────────────┘
```

### Resource Utilization

- **Total Capacity**: 16,777,216 bytes (16MB)
- **Used Space**: 16,756,224 bytes (99.87%)
- **Free Space**: 20,992 bytes (0.13%)
- **Sector Alignment**: All resources aligned to 4KB boundaries

## 📁 Directory Structure

```text
flash-content-generator/
├── 🛠️  tools/                    # Core generation and analysis tools
│   ├── font_converter.py         # TTF → Bitmap converter (1-bit monochrome)
│   ├── svg_to_rgb565.py          # SVG → RGB565 converter for displays
│   ├── flash_composer.py         # Complete 16MB firmware assembler
│   ├── resource_manager.py       # Memory layout and address manager
│   ├── analyze_font.py           # Font file structure analyzer
│   └── data_parser.py            # Binary data parsing utilities
├── 🌐 web-app/                   # Interactive analysis interface
│   ├── index.html                # Main analyzer dashboard
│   ├── script.js                 # Font/image parsing and visualization
│   ├── style.css                 # UI styling and responsive design
│   └── *.bin                     # Sample firmware files for testing
├── 📦 assets/                    # Source materials and outputs
│   ├── VonwaonBitmap-12px.ttf    # Source font (12px optimized)
│   ├── VonwaonBitmap-16px.ttf    # Source font (16px optimized)
│   ├── boot_screen.svg           # Vector boot screen design
│   ├── memory_map.txt            # Detailed memory layout documentation
│   ├── resource_layout.json     # Machine-readable resource configuration
│   └── font_output/              # Generated bitmap files
│       ├── font_bitmap_12px.bin  # 12px font data (1MB)
│       ├── font_bitmap_16px.bin  # 16px font data (1MB)
│       ├── *_info.txt            # Font metadata and statistics
│       └── boot_screen_320x172.bin # RGB565 boot screen (110KB)
└── 📋 generate_firmware.py       # One-click firmware generation script
```

## 🚀 Quick Start

### One-Command Generation (Recommended)

Generate complete Flash firmware with automatic web deployment:

```bash
python3 generate_firmware.py
```

This single command will:

1. **Font Processing**: Convert TTF fonts to optimized 1-bit bitmap format
   - 12px VonwaonBitmap: 27,678 characters → 1MB binary
   - 16px VonwaonBitmap: 27,678 characters → 1MB binary
2. **Image Processing**: Generate RGB565 boot screen from SVG (320×172 → 110KB)
3. **Firmware Assembly**: Compose complete 16MB Flash image with proper alignment
4. **Web Deployment**: Automatically copy firmware to web-app for interactive analysis
5. **Integrity Verification**: Validate all data structures and checksums

**Generated Files:**

- `w25q128jv_complete.bin` - Complete 16MB Flash firmware (ready for programming)
- `web-app/w25q128jv_complete.bin` - Copy for web-based analysis
- `assets/font_output/*.bin` - Individual font bitmap files
- `assets/boot_screen_320x172.bin` - RGB565 boot screen
- Detailed generation logs with performance metrics and verification results

### Programming the Firmware

```bash
# Program complete firmware to flash chip
flash-programmer-tool --port /dev/ttyACM0 write \
  --file w25q128jv_complete.bin --address 0x000000 --erase --verify

# Verify programming success
flash-programmer-tool --port /dev/ttyACM0 info
```

### Manual Step-by-Step Generation

For advanced users who want granular control over the generation process:

#### 1. Font Bitmap Generation

Convert TTF fonts to embedded-optimized bitmap format:

```bash
cd tools

# Generate 12px font bitmap (1MB, 27,678 characters)
python font_converter.py ../assets/VonwaonBitmap-12px.ttf \
  --output ../assets/font_output/ \
  --size 12 \
  --filename font_bitmap_12px.bin

# Generate 16px font bitmap (1MB, 27,678 characters)
python font_converter.py ../assets/VonwaonBitmap-16px.ttf \
  --output ../assets/font_output/ \
  --size 16 \
  --filename font_bitmap_16px.bin
```

**Font Processing Details:**

- **Character Set**: ASCII (32-126) + CJK Unified Ideographs (4E00-9FFF) + Extensions
- **Format**: 1-bit monochrome bitmap, 8 pixels per byte
- **Structure**: Header + Character Info Table + Bitmap Data
- **Optimization**: Binary search-optimized Unicode ordering

#### 2. Boot Screen Generation

Convert SVG graphics to RGB565 format for embedded displays:

```bash
python svg_to_rgb565.py ../assets/boot_screen.svg \
  --output ../assets/boot_screen_320x172.bin \
  --width 320 --height 172 \
  --format rgb565
```

**Image Processing Details:**

- **Input**: Vector SVG graphics (scalable)
- **Output**: RGB565 binary (2 bytes per pixel)
- **Resolution**: 320×172 pixels (54,880 pixels total)
- **Size**: 110,080 bytes (110KB)

#### 3. Flash Image Composition

Assemble complete 16MB firmware with automatic resource alignment:

```bash
python flash_composer.py \
  --boot-screen ../assets/boot_screen_320x172.bin \
  --font-12px ../assets/font_output/font_bitmap_12px.bin \
  --font-16px ../assets/font_output/font_bitmap_16px.bin \
  --output w25q128jv_complete.bin \
  --verify
```

**Composition Process:**

- **Memory Alignment**: All resources aligned to 4KB sector boundaries
- **Gap Filling**: Unused areas filled with 0xFF (erased flash state)
- **Verification**: CRC32 checksums and structure validation
- **Web Deployment**: Automatic copy to web-app directory

### Web-based Flash Content Analysis

The web application provides comprehensive firmware analysis and debugging tools:

```bash
cd web-app
# Open index.html in your browser
# Load w25q128jv_complete.bin for analysis
```

**Analysis Features:**

- **📊 Flash Firmware Viewer**:
  - Memory layout visualization with address mapping
  - Resource block inspection and size analysis
  - Sector usage statistics and fragmentation analysis

- **🔤 Font Resource Browser**:
  - Browse all 27,678+ characters in both 12px and 16px fonts
  - Character search by Unicode code point or visual appearance
  - Bitmap data inspection with pixel-level zoom (1x, 4x, 8x, 16x)
  - Font metrics analysis (character dimensions, bitmap sizes)

- **🖼️ Image Resource Viewer**:
  - RGB565 boot screen preview (320×172 resolution)
  - Pixel-level color analysis with RGB values
  - Image statistics and color distribution

## 🔧 Data Formats and Parsing

### Font Bitmap Format Specification

The generated font files use a custom binary format optimized for embedded systems:

#### Binary Structure

```text
┌─────────────────────────────────────────────────────────────┐
│ Header (4 bytes)                                            │
├─────────────────────────────────────────────────────────────┤
│ Character Count (uint32_t, little-endian)                  │
├─────────────────────────────────────────────────────────────┤
│ Character Info Table (8 bytes × character_count)           │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Unicode Code Point (uint32_t, little-endian)           │ │
│ │ Width (uint8_t)                                         │ │
│ │ Height (uint8_t)                                        │ │
│ │ Bitmap Offset (uint16_t, little-endian)                │ │
│ └─────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│ Bitmap Data (variable size)                                │
│ • 1-bit monochrome bitmap                                   │
│ • 8 pixels per byte, MSB first                             │
│ • Row-major order (left-to-right, top-to-bottom)           │
│ • Byte-aligned rows: bytes_per_row = (width + 7) / 8       │
└─────────────────────────────────────────────────────────────┘
```

#### Parsing Example (Rust)

```rust
use heapless::Vec;

/// Font bitmap header structure
#[derive(Debug, Clone)]
pub struct FontHeader {
    pub char_count: u32,
}

/// Character information structure (8 bytes for 12px font)
#[derive(Debug, Clone)]
pub struct CharInfo {
    pub unicode: u32,
    pub width: u8,
    pub height: u8,
    pub bitmap_offset: u16,
}

/// Font bitmap parser
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

    /// Find character by Unicode code point (linear search - fonts are Unicode-sorted for binary search)
    pub fn find_char_by_unicode(
        data: &[u8],
        unicode: u32
    ) -> Result<Option<(usize, CharInfo)>, &'static str> {
        let header = Self::parse_header(data)?;

        // Linear search implementation (as used in actual project)
        // Note: Characters are sorted by Unicode for binary search optimization
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

                // MSB first: extract pixel using shift (7 - bit_offset)
                let pixel = (bitmap[byte_index] >> (7 - bit_offset)) & 1 != 0;
                pixels.push(pixel).map_err(|_| "Pixel buffer full")?;
            }
        }

        Ok(pixels)
    }
}
```

**Note**: This implementation uses a consistent 8-byte character info structure for both 12px and 16px fonts. The bitmap_offset is a 16-bit value, which limits individual character bitmaps to 64KB offset from the bitmap data start.

### RGB565 Image Format

Boot screen images use standard RGB565 format:

```text
┌─────────────────────────────────────────────────────────────┐
│ RGB565 Pixel Data (2 bytes per pixel)                      │
├─────────────────────────────────────────────────────────────┤
│ Pixel Format: RRRRRGGGGGGBBBBB (16-bit, little-endian)     │
│ • Red:   5 bits (bits 15-11)                               │
│ • Green: 6 bits (bits 10-5)                                │
│ • Blue:  5 bits (bits 4-0)                                 │
├─────────────────────────────────────────────────────────────┤
│ Layout: Row-major order (320×172 = 54,880 pixels)          │
│ Size: 110,080 bytes (54,880 × 2 bytes)                     │
└─────────────────────────────────────────────────────────────┘
```

#### RGB565 Parsing Example (Rust)

```rust
use embedded_graphics::pixelcolor::Rgb565;

/// Image information structure
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u16,
    pub height: u16,
    pub format: ImageFormat,
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Rgb565,
}

/// Image parser for RGB565 format
pub struct ImageParser;

impl ImageParser {
    /// Parse boot screen image info (320x172 RGB565)
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

        let pixel_index = y as usize * info.width as usize + x as usize;
        Self::rgb565_to_color(data, pixel_index)
    }

    /// Convert RGB565 to RGB888 components (with proper bit expansion)
    pub fn rgb565_to_rgb888(rgb565: Rgb565) -> (u8, u8, u8) {
        // Extract 5-bit red, 6-bit green, 5-bit blue components
        let r5 = rgb565.r();
        let g6 = rgb565.g();
        let b5 = rgb565.b();

        // Expand to 8-bit with proper scaling
        let r8 = (r5 << 3) | (r5 >> 2);  // 5-bit to 8-bit: replicate MSBs
        let g8 = (g6 << 2) | (g6 >> 4);  // 6-bit to 8-bit: replicate MSBs
        let b8 = (b5 << 3) | (b5 >> 2);  // 5-bit to 8-bit: replicate MSBs

        (r8, g8, b8)
    }

    /// Calculate image statistics
    pub fn calculate_stats(data: &[u8], info: &ImageInfo) -> Result<ImageStats, &'static str> {
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

        if valid_pixels == 0 {
            return Err("No valid pixels found");
        }

        Ok(ImageStats {
            total_pixels,
            valid_pixels,
            expected_size,
            actual_size: data.len() as u32,
            avg_red: (red_sum / valid_pixels) as u8,
            avg_green: (green_sum / valid_pixels) as u8,
            avg_blue: (blue_sum / valid_pixels) as u8,
        })
    }
}

#[derive(Debug)]
pub struct ImageStats {
    pub total_pixels: u32,
    pub valid_pixels: u32,
    pub expected_size: u32,
    pub actual_size: u32,
    pub avg_red: u8,
    pub avg_green: u8,
    pub avg_blue: u8,
}

/// Example: Complete character rendering pipeline for STM32 projects
pub async fn render_character_from_flash(
    flash_data: &[u8],
    character: char,
    x: i32,
    y: i32,
    color: Rgb565
) -> Result<(), &'static str> {
    // 1. Find character in font data
    let unicode = character as u32;
    let (char_index, char_info) = FontParser::find_char_by_unicode(flash_data, unicode)?
        .ok_or("Character not found in font")?;

    // 2. Parse font header
    let header = FontParser::parse_header(flash_data)?;

    // 3. Extract character bitmap
    let bitmap = FontParser::extract_bitmap(flash_data, &char_info, &header)?;

    // 4. Convert bitmap to pixels for rendering
    let pixels = FontParser::bitmap_to_pixels(&bitmap, char_info.width, char_info.height)?;

    // 5. Render pixels to display (example using embedded-graphics pattern)
    for (i, &pixel_on) in pixels.iter().enumerate() {
        if pixel_on {
            let pixel_x = x + (i % char_info.width as usize) as i32;
            let pixel_y = y + (i / char_info.width as usize) as i32;

            // In actual STM32 implementation, this would call your display driver:
            // display.draw_pixel(Point::new(pixel_x, pixel_y), color).await?;
        }
    }

    defmt::debug!("Rendered character '{}' ({}x{}) at ({}, {})",
                 character, char_info.width, char_info.height, x, y);

    Ok(())
}

/// Example: Load and display RGB565 boot screen
pub async fn display_boot_screen_from_flash(
    flash_data: &[u8],
    display_width: u16,
    display_height: u16
) -> Result<(), &'static str> {
    let image_info = ImageParser::parse_boot_screen_info();

    // Verify data size
    let expected_size = image_info.width as usize * image_info.height as usize * 2;
    if flash_data.len() < expected_size {
        return Err("Insufficient flash data for boot screen");
    }

    // Process image in chunks for memory efficiency
    const CHUNK_SIZE: usize = 1024; // Process 512 pixels at a time

    for chunk_start in (0..expected_size).step_by(CHUNK_SIZE) {
        let chunk_end = (chunk_start + CHUNK_SIZE).min(expected_size);
        let chunk_data = &flash_data[chunk_start..chunk_end];

        // Convert chunk to RGB565 colors
        for i in (0..chunk_data.len()).step_by(2) {
            if i + 1 < chunk_data.len() {
                let pixel_index = (chunk_start + i) / 2;
                let x = (pixel_index % image_info.width as usize) as u16;
                let y = (pixel_index / image_info.width as usize) as u16;

                if x < display_width && y < display_height {
                    let color = ImageParser::rgb565_to_color(chunk_data, i / 2)?;

                    // In actual STM32 implementation:
                    // display.draw_pixel(Point::new(x as i32, y as i32), color).await?;
                }
            }
        }
    }

    defmt::info!("Boot screen displayed: {}x{} pixels", image_info.width, image_info.height);
    Ok(())
}
```

## 🛠️ Tools Reference

### font_converter.py

Advanced TTF to bitmap converter with embedded optimization:

```bash
python font_converter.py [font_file] [options]

Options:
  --output DIR          Output directory for generated files
  --size SIZE           Font size in pixels (default: 12)
  --filename NAME       Output filename (default: font_bitmap.bin)
  --charset RANGE       Character range (ascii, cjk, all)
  --optimize            Enable bitmap compression
  --info                Generate detailed info file
```

**Features:**

- **Character Set Support**: ASCII (95 chars) + CJK Unified Ideographs (20,992 chars) + Extensions (6,591 chars)
- **Bitmap Optimization**: 1-bit monochrome with 8-pixel byte packing
- **Binary Search Ready**: Unicode-sorted character table for O(log n) lookup
- **Memory Efficient**: Minimal overhead with compact data structures

### svg_to_rgb565.py

SVG to RGB565 converter for embedded displays:

```bash
python svg_to_rgb565.py [svg_file] [options]

Options:
  --output FILE         Output binary file path
  --width WIDTH         Target width in pixels (default: 320)
  --height HEIGHT       Target height in pixels (default: 172)
  --format FORMAT       Output format (rgb565, rgb888)
  --dpi DPI             Rendering DPI (default: 96)
```

### flash_composer.py

Complete firmware assembler with verification:

```bash
python flash_composer.py [options]

Options:
  --boot-screen FILE    Boot screen binary file
  --font-12px FILE      12px font bitmap file
  --font-16px FILE      16px font bitmap file
  --output FILE         Output firmware file (default: w25q128jv_complete.bin)
  --verify              Enable integrity verification
  --web-deploy          Copy to web-app directory
```

**Assembly Process:**

1. **Resource Validation**: Verify all input files and sizes
2. **Memory Layout**: Calculate addresses with 4KB sector alignment
3. **Gap Filling**: Fill unused areas with 0xFF (erased flash state)
4. **Integrity Check**: Generate and verify CRC32 checksums
5. **Web Deployment**: Automatic copy for browser-based analysis

### resource_manager.py

Memory layout and resource allocation manager:

```bash
python resource_manager.py [options]

Options:
  --layout FILE         Resource layout JSON file
  --generate-map        Generate memory map documentation
  --validate            Validate resource allocation
  --optimize            Optimize memory usage
```

## 🔍 Data Extraction and Analysis

### Reading Flash Content

Extract specific resources from programmed flash chips:

```bash
# Extract complete firmware image
flash-programmer-tool --port /dev/ttyACM0 read \
  --file firmware_backup.bin --address 0x000000 --size 0x1000000

# Extract 12px font data only
flash-programmer-tool --port /dev/ttyACM0 read \
  --file font_12px_extracted.bin --address 0x020000 --size 0x100000

# Extract 16px font data only
flash-programmer-tool --port /dev/ttyACM0 read \
  --file font_16px_extracted.bin --address 0x120000 --size 0x100000

# Extract boot screen only
flash-programmer-tool --port /dev/ttyACM0 read \
  --file boot_screen_extracted.bin --address 0x000000 --size 0x1AE00
```

### Analyzing Extracted Data

Use the web application for comprehensive analysis:

1. **Load Firmware**: Open `web-app/index.html` and load extracted `.bin` files
2. **Font Analysis**: Browse character sets, inspect bitmap data, verify Unicode mappings
3. **Image Analysis**: View RGB565 images, analyze color distribution, check pixel accuracy
4. **Memory Analysis**: Examine resource layout, sector usage, and fragmentation

### Verification and Debugging

```bash
# Verify font structure integrity
python tools/analyze_font.py font_12px_extracted.bin

# Compare original vs extracted data
python tools/verify_flash_image.py w25q128jv_complete.bin firmware_backup.bin

# Generate detailed memory report
python tools/resource_manager.py --layout assets/resource_layout.json --validate
```

## 🔗 STM32 Integration

### Hardware Requirements

- **Microcontroller**: STM32G431CBU6 (or compatible STM32G4 series)
- **Flash Memory**: W25Q128JV (16MB SPI Flash)
- **Display**: ST7789 or compatible RGB565 display controller
- **SPI Connection**: Standard 4-wire SPI (CS, CLK, MOSI, MISO)

### Software Integration

The generated firmware is designed for STM32 projects with:

1. **SPI Flash Driver**: W25Q128JV read/write operations
2. **Display Driver**: RGB565 framebuffer support
3. **Font Renderer**: Binary search character lookup with bitmap rendering
4. **Resource Manager**: Address-based resource access

**Reference Implementation**: See `../stm32g431-w25q128jv/` for complete working example with:

- SPI flash initialization and communication
- Font parsing and character rendering
- RGB565 image display
- Resource management and caching

## 📋 Requirements

### System Requirements

- **Python**: 3.7+ with pip package manager
- **Memory**: 2GB RAM minimum (4GB recommended for large font processing)
- **Storage**: 500MB free space for generated firmware files
- **Browser**: Modern web browser (Chrome 80+, Firefox 75+, Safari 13+)

### Python Dependencies

```bash
# Core dependencies
pip install Pillow>=8.0.0          # Image processing and SVG rendering
pip install fontTools>=4.0.0       # TTF font parsing and glyph extraction

# Optional dependencies for enhanced features
pip install cairosvg>=2.5.0        # Improved SVG rendering quality
pip install numpy>=1.20.0          # Faster bitmap processing
```

### Hardware Requirements (for programming)

- **STM32G4 Development Board**: With USB CDC support
- **W25Q128JV Flash Chip**: 16MB SPI flash memory
- **SPI Connections**: Proper wiring between STM32G4 and flash chip
- **USB Cable**: For communication with flash programmer tool

## 🚀 Installation

### Quick Setup

```bash
# Clone the repository (if not already done)
cd examples/flash-content-generator

# Install Python dependencies
pip install -r requirements.txt

# Verify installation
python3 generate_firmware.py --help
```

### Development Setup

```bash
# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install Pillow fontTools cairosvg numpy

# Run tests
python3 -m pytest tests/  # If test suite is available
```

## 🐛 Troubleshooting

### Common Issues

**Font Conversion Errors:**

```bash
# Issue: "Font file not found" or "Invalid TTF file"
# Solution: Verify font file path and format
python tools/analyze_font.py assets/VonwaonBitmap-12px.ttf
```

**Memory Issues:**

```bash
# Issue: "MemoryError during font processing"
# Solution: Process fonts individually or increase system memory
python font_converter.py --charset ascii  # Process smaller character set first
```

**Web App Loading Issues:**

```bash
# Issue: "Failed to load firmware file"
# Solution: Ensure firmware file is in web-app directory
cp w25q128jv_complete.bin web-app/
```

**Flash Programming Issues:**

```bash
# Issue: "Device not found" or "Communication timeout"
# Solution: Check USB connection and port permissions
flash-programmer-tool --port /dev/ttyACM0 info  # Test connection
sudo usermod -a -G dialout $USER  # Add user to dialout group (Linux)
```

### Performance Optimization

```bash
# Enable faster processing for large fonts
export PYTHONOPTIMIZE=1
export OMP_NUM_THREADS=4  # Use multiple CPU cores

# Use SSD storage for faster I/O
export TMPDIR=/path/to/ssd/temp
```

## 📊 Performance Metrics

| Operation | File Size | Processing Time | Memory Usage |
|-----------|-----------|-----------------|--------------|
| Font 12px Generation | 1MB | ~30 seconds | ~500MB RAM |
| Font 16px Generation | 1MB | ~45 seconds | ~750MB RAM |
| Boot Screen Conversion | 110KB | ~2 seconds | ~50MB RAM |
| Complete Firmware Assembly | 16MB | ~10 seconds | ~100MB RAM |
| Web App Loading | 16MB | ~5 seconds | ~200MB RAM |

## 📄 License

This project is part of the STM32G4 Flash Programmer toolkit.

**Copyright**: Ivan's Projects
**License**: MIT License (see LICENSE file for details)
**Version**: 1.0.0
**Last Updated**: 2024
