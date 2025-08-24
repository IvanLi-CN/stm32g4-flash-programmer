# Flash Content Generator

A comprehensive toolchain for generating, managing, and analyzing content stored in external SPI Flash memory for STM32G431CBU6 embedded projects. This system creates optimized firmware images containing fonts, graphics, and application data for W25Q128JV (16MB) flash chips.

## 🎯 Overview

The Flash Content Generator provides tools for:

- **CJK Font Processing**: Converting TTF fonts to optimized 1-bit bitmap format with 27,678+ characters (ASCII + CJK)
- **Custom Font Generation**: Creating specialized monospace fonts for embedded displays (digit and ASCII fonts)
- **Image Processing**: Converting SVG graphics to RGB565 format for embedded displays
- **Memory Management**: Creating structured Flash layouts with automatic address allocation
- **Firmware Composition**: Assembling complete 16MB flash images with multiple resource types
- **Data Verification**: Comprehensive validation and integrity checking
- **Web-based Analysis**: Interactive tools for inspecting and debugging flash content

## 🏗️ Flash Memory Layout (W25Q128JV - 16MB)

```text
┌─────────────────────────────────────────────────────────────┐
│ 0x000000 │ Boot Screen      │ 110KB  │ RGB565 320×172      │
├─────────────────────────────────────────────────────────────┤
│ 0x020000 │ CJK Font 12px    │ 1MB    │ 27,678 characters   │
├─────────────────────────────────────────────────────────────┤
│ 0x120000 │ CJK Font 16px    │ 1MB    │ 27,678 characters   │
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
│ 0x7D0000 │ Digit Font 24×48 │ 2KB    │ 12 characters       │
├─────────────────────────────────────────────────────────────┤
│ 0x7D1000 │ ASCII Font 16×24 │ 6KB    │ 95 characters       │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- Python 3.7+
- PIL/Pillow library: `pip install Pillow`

### 5-Minute Setup

1. **Generate Font Files**

   ```bash
   cd flash-programmer-reference/examples/flash-content-generator

   # Generate all fonts (recommended)
   python font_tools.py generate

   # Or build complete flash image
   python font_tools.py build
   ```

2. **View Generated Fonts**

   ```bash
   # View digit font information
   python font_tools.py view output/digit_font_24x48.bin --info

   # View ASCII font and render character 'A'
   python font_tools.py view output/ascii_font_16x24.bin --render 33
   ```

3. **Verify Flash Image**

   ```bash
   # Verify fonts are correctly embedded in flash image
   python font_tools.py verify pd-sink-128mbit.bin
   ```

### Output Files

After execution, you will get:

```text
📁 flash-programmer-reference/examples/flash-content-generator/
├── pd-sink-128mbit.bin              # ✅ Final Flash image (16MB)
├── output/
│   ├── digit_font_24x48.bin         # ✅ 24×48 digit font
│   └── ascii_font_16x24.bin         # ✅ 16×24 ASCII font
└── assets/font_output/
    ├── digit_font_24x48.bin         # ✅ Font copy
    └── ascii_font_16x24.bin         # ✅ Font copy
```

## 📁 Directory Structure

```text
flash-content-generator/
├── 🛠️  tools/                    # Core generation and analysis tools
│   ├── font_converter.py         # TTF → Bitmap converter (1-bit monochrome)
│   ├── custom_font_generator.py  # Custom monospace font generator
│   ├── font_viewer.py            # Font file inspector and validator
│   ├── svg_to_rgb565.py          # SVG → RGB565 converter for displays
│   ├── flash_composer.py         # Complete 16MB firmware assembler
│   ├── resource_manager.py       # Memory layout and address manager
│   └── verify_fonts_in_flash.py  # Flash verification tool
├── 🌐 web-app/                   # Interactive analysis interface
│   ├── index.html                # Main analyzer dashboard
│   ├── script.js                 # Font/image parsing and visualization
│   └── *.bin                     # Sample firmware files for testing
├── 📦 assets/                    # Source materials and outputs
│   ├── VonwaonBitmap-12px.ttf    # Source CJK font (12px)
│   ├── VonwaonBitmap-16px.ttf    # Source CJK font (16px)
│   ├── boot_screen.svg           # Vector boot screen design
│   ├── memory_map.txt            # Detailed memory layout documentation
│   ├── resource_layout.json     # Machine-readable resource configuration
│   └── font_output/              # Generated bitmap files
├── 📋 generate_firmware.py       # One-click firmware generation script
└── 🔧 font_tools.py              # Unified CLI tool
```

## 🔧 Custom Fonts

### Generated Font Types

1. **Digital Font (24×48 pixels)**
   - **Purpose**: Large numeric display for voltage, current, power readings
   - **Characters**: `0123456789-.` (12 characters total)
   - **Flash Address**: `0x7D0000`
   - **File Size**: ~2KB

2. **ASCII Font (16×24 pixels)**
   - **Purpose**: General text display for menus, labels, status messages
   - **Characters**: ASCII 32-126 (95 printable characters)
   - **Flash Address**: `0x7D1000`
   - **File Size**: ~6KB

### Font Data Format

Both fonts use a binary format compatible with embedded systems:

```text
[4 bytes] Character count (little-endian)
[N×8 bytes] Character information array:
  - [4 bytes] Unicode code point (little-endian)
  - [1 byte] Character width
  - [1 byte] Character height
  - [2 bytes] Bitmap data offset (little-endian)
[Variable] Bitmap data (1 bit per pixel, packed by bytes)
```

## 🛠️ Tools Reference

### font_tools.py - Unified CLI Tool

The main interface for all font operations:

```bash
# Generate custom fonts
python font_tools.py generate [options]

# View font file information
python font_tools.py view <font_file> [options]

# Verify fonts in flash image
python font_tools.py verify <flash_file>

# Build complete flash image
python font_tools.py build
```

**Common Commands:**

```bash
# Generate all custom fonts
python font_tools.py generate

# Generate only digit font with specific font
python font_tools.py generate --digit-only --font-name "Consolas"

# View font file information
python font_tools.py view output/digit_font_24x48.bin --info

# View character table and render specific character
python font_tools.py view output/ascii_font_16x24.bin --table 10 --render 65

# Verify fonts in flash image
python font_tools.py verify pd-sink-128mbit.bin
```

### Individual Tools

- **`custom_font_generator.py`** - Standalone generator for custom digit and ASCII fonts
- **`font_viewer.py`** - Font file inspector and validator with ASCII art rendering
- **`font_converter.py`** - TTF to bitmap converter for large CJK fonts (27,678 characters)
- **`svg_to_rgb565.py`** - SVG to RGB565 converter for boot screens
- **`flash_composer.py`** - Complete firmware assembler
- **`verify_fonts_in_flash.py`** - Flash verification tool

### Manual Generation Steps

1. **Generate Custom Fonts**

   ```bash
   cd tools
   python custom_font_generator.py --output-dir ../output
   ```

2. **Generate Boot Screen**

   ```bash
   python svg_to_rgb565.py ../assets/boot_screen.svg \
     --output ../assets/boot_screen_320x172.bin \
     --width 320 --height 172
   ```

3. **Compose Flash Image**

   ```bash
   python flash_composer.py \
     --boot-screen ../assets/boot_screen_320x172.bin \
     --output pd-sink-128mbit.bin \
     --verify
   ```

## 🌐 Web-based Analysis

The web application provides comprehensive firmware analysis and debugging tools:

```bash
cd web-app
python start_server.py
# Browser opens automatically at http://localhost:8080
```

**Analysis Features:**

- **📊 Flash Firmware Viewer**:
  - Memory layout visualization with address mapping
  - Resource block inspection and size analysis
  - Sector usage statistics and fragmentation analysis

- **🔤 Font Resource Browser**:
  - Browse all characters in CJK fonts (12px/16px) and custom fonts (digit/ASCII)
  - Character search by Unicode code point or visual appearance
  - Bitmap data inspection with pixel-level zoom (1x, 4x, 8x, 16x)
  - Font metrics analysis (character dimensions, bitmap sizes)

- **🖼️ Image Resource Viewer**:
  - RGB565 boot screen preview (320×172 resolution)
  - Pixel-level color analysis with RGB values
  - Image statistics and color distribution

### Supported Font Types

#### CJK Fonts (Large Character Sets)

- **12px CJK Font**:
  - **Location**: 0x020000 (Flash address)
  - **Size**: 1MB
  - **Characters**: WenQuanYi bitmap font with 27,678 characters (ASCII + CJK Unified + Extensions)
  - **File**: `font_bitmap_12px.bin`

- **16px CJK Font**:
  - **Location**: 0x120000 (Flash address)
  - **Size**: 1MB
  - **Characters**: WenQuanYi bitmap font with 27,678 characters (ASCII + CJK Unified + Extensions)
  - **File**: `font_bitmap_16px.bin`

#### Custom Fonts (Embedded Optimized)

- **24×48 Digital Font**:
  - **Location**: 0x7D0000 (Flash address)
  - **Size**: ~2KB
  - **Characters**: Monospace numbers 0-9, minus, decimal point (12 characters)
  - **File**: `digit_font_24x48.bin`

- **16×24 ASCII Font**:
  - **Location**: 0x7D1000 (Flash address)
  - **Size**: ~6KB
  - **Characters**: Complete ASCII printable set 32-126 (95 characters)
  - **File**: `ascii_font_16x24.bin`

## 🔧 Troubleshooting

### Common Issues

**Font Generation Errors:**

```bash
# Issue: "Font file not found" or "Invalid TTF file"
# Solution: Verify font file path and format
python tools/font_viewer.py output/digit_font_24x48.bin --info
```

**Memory Issues:**

```bash
# Issue: "MemoryError during font processing"
# Solution: Process fonts individually or increase system memory
python font_tools.py generate --digit-only  # Process smaller character set first
```

**Web App Loading Issues:**

```bash
# Issue: "Failed to load firmware file"
# Solution: Ensure firmware file is in web-app directory
cp pd-sink-128mbit.bin web-app/
```

**Flash Programming Issues:**

```bash
# Issue: "Device not found" or "Communication timeout"
# Solution: Check USB connection and port permissions
# Test connection with your flash programmer tool
```

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

### Installation

```bash
# Clone the repository (if not already done)
cd flash-programmer-reference/examples/flash-content-generator

# Install Python dependencies
pip install Pillow fontTools

# Verify installation
python font_tools.py --help
```

## 📄 License

This project is part of the STM32G431CBU6 Flash Programmer toolkit.

**Copyright**: Ivan's Projects
**License**: MIT License
**Version**: 1.0.0
