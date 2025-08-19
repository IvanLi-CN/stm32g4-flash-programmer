# Flash Content Generator

This directory contains tools and utilities for generating content to be stored in external Flash memory for STM32 projects.

## Overview

The Flash Content Generator provides a complete toolchain for:

- Converting fonts to bitmap format suitable for embedded displays
- Generating boot screen images in RGB565 format
- Creating Flash memory layouts and resource maps
- Analyzing and verifying Flash content
- Web-based font viewer and analyzer

## Directory Structure

```text
flash-content-generator/
├── tools/                  # Python scripts for content generation
│   ├── font_converter.py   # Convert TTF fonts to bitmap format
│   ├── svg_to_rgb565.py    # Convert SVG to RGB565 boot screen
│   ├── flash_composer.py   # Compose complete Flash image
│   ├── resource_manager.py # Manage Flash resource layout
│   ├── analyze_font.py     # Analyze font files
│   ├── verify_flash_image.py # Verify Flash content
│   └── check_flash_address.py # Check Flash addresses
├── web-app/                # Web-based font viewer and analyzer
│   ├── index.html          # Font bitmap analyzer interface
│   ├── script.js           # JavaScript for font analysis
│   └── *.bin               # Sample font bitmap files
└── assets/                 # Source assets and generated content
    ├── VonwaonBitmap-*.ttf # Source font files
    ├── boot_screen.svg     # Source boot screen design
    ├── memory_map.txt      # Flash memory layout
    ├── resource_layout.json # Resource configuration
    └── font_output/        # Generated font bitmaps
```

## Quick Start

### One-Command Generation (Recommended)

Generate complete Flash firmware with automatic web deployment:

```bash
python3 generate_firmware.py
```

This single command will:

1. Convert TTF fonts to bitmap format (12px + 16px)
2. Generate RGB565 boot screen from SVG
3. Compose complete 16MB Flash image
4. Automatically deploy to web-app for preview
5. Verify Flash image integrity

**Output:**

- `w25q128jv_complete.bin` - Complete 16MB Flash image
- `web-app/w25q128jv_complete.bin` - Copy for web preview
- Detailed generation log and verification report

### Manual Step-by-Step Generation

For advanced users who want granular control:

#### 1. Font Generation

Convert a TTF font to bitmap format:

```bash
cd tools
python font_converter.py ../assets/VonwaonBitmap-12px.ttf --output ../assets/font_output/
```

#### 2. Boot Screen Generation

Convert SVG to RGB565 format:

```bash
python svg_to_rgb565.py ../assets/boot_screen.svg --output ../assets/boot_screen_320x172.bin --width 320 --height 172
```

#### 3. Flash Image Composition

Create complete Flash image (automatically copies to web-app for preview):

```bash
python flash_composer.py
```

### Web-based Flash Content Preview

The web app automatically loads the generated firmware for preview:

```bash
cd web-app
# Open index.html in your browser
# The generated firmware is automatically loaded
# Switch between Flash firmware, Font resources, and Image resources
```

**Features:**

- **Flash Firmware**: View memory layout and resource blocks
- **Font Resources**: Browse 12px/16px fonts with 27,678+ characters
- **Image Resources**: Preview RGB565 boot screen (320×172)

## Tools Description

### font_converter.py

Converts TTF fonts to bitmap format suitable for embedded displays. Supports:

- Configurable font sizes
- Character subset selection (ASCII, CJK, etc.)
- Multiple output formats
- Bitmap optimization

### svg_to_rgb565.py

Converts SVG graphics to RGB565 binary format for boot screens:

- Configurable output dimensions
- RGB565 color space conversion
- Optimized for embedded displays

### flash_composer.py

Composes complete Flash images from multiple resources:

- Font bitmaps
- Boot screen images
- Configuration data
- Memory layout management
- **Automatic web-app deployment** (copies firmware for preview)

### resource_manager.py

Manages Flash memory layout and resource allocation:

- Address calculation
- Resource mapping
- Layout validation
- Memory usage optimization

### Web App Features

- Upload and analyze font bitmap files
- Character-by-character visualization
- Bitmap data inspection
- Font metrics analysis
- Export individual characters

## Integration with STM32 Projects

The generated content is designed to work with STM32 projects that:

1. Use external SPI Flash memory (e.g., W25Q128JV)
2. Have display controllers (e.g., ST7789)
3. Implement Flash-based font rendering

See the `../stm32g431-w25q128jv/` example for a complete implementation.

## Requirements

- Python 3.7+
- PIL (Pillow) for image processing
- fontTools for TTF font parsing
- Modern web browser for the web app

## Installation

```bash
pip install Pillow fontTools
```

## License

This project is part of the STM32G4 Flash Programmer toolkit.
