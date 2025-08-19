# STM32G4 Flash Programmer Tool

A high-performance command-line tool for programming external SPI flash memory chips through STM32G4 microcontrollers. This tool provides a complete interface for reading, writing, erasing, and verifying W25Q128JV flash memory via USB CDC communication.

## ✨ Features

- 🚀 **High-speed USB CDC communication** with automatic flow control
- 💾 **Complete SPI flash support** (W25Q128JV tested, compatible with similar chips)
- 🔄 **Automatic erase before write** with `--erase` flag
- ✅ **Progressive CRC32 verification** for data integrity
- 🛡️ **Hardware-accelerated operations** via STM32G4 firmware
- 📊 **Real-time progress indicators** with speed and ETA
- 🎯 **Flexible addressing** with hex format support
- 🔧 **Multiple write modes** (basic, stream, batch)

## 🔧 Hardware Requirements

- **STM32G431CBU6** microcontroller (or compatible STM32G4 series)
- **W25Q128JV** SPI flash memory (16MB)
- **USB connection** for communication
- **SPI wiring** between STM32G4 and flash chip

### SPI Connection Diagram

```text
STM32G431CBU6          W25Q128JV Flash
┌─────────────┐       ┌─────────────┐
│ PB12 (NSS)  │────── │ CS          │
│ PB13 (SCK)  │────── │ CLK         │
│ PB14 (MISO) │────── │ DO          │
│ PB15 (MOSI) │────── │ DI          │
│ 3.3V        │────── │ VCC         │
│ GND         │────── │ GND         │
└─────────────┘       └─────────────┘
```

## 🚀 Quick Start

### 1. Build the Tool

```bash
cd host-tool
cargo build --release
```

### 2. Flash STM32G4 Firmware

First, program the STM32G4 with the flash programmer firmware:

```bash
cd ../firmware
cargo run --release
```

### 3. Test Connection

```bash
cd ../host-tool
./target/release/flash-programmer-tool --port /dev/ttyACM0 info
```

## 📖 Usage Examples

### 🔍 Get Flash Information

```bash
flash-programmer-tool --port /dev/ttyACM0 info
```

**Output:**

```text
Flash Information:
  JEDEC ID: 0xEF4018
  Total Size: 16 MB (16777216 bytes)
  Page Size: 256 bytes
  Sector Size: 4 KB (4096 bytes)
```

### 📖 Read Flash Memory

```bash
# Read 1KB from address 0x0
flash-programmer-tool --port /dev/ttyACM0 read \
  --file backup.bin --address 0x0 --size 0x400

# Read entire flash (16MB) - takes ~15 minutes
flash-programmer-tool --port /dev/ttyACM0 read \
  --file full_backup.bin --address 0x0 --size 0x1000000
```

### ✍️ Write Flash Memory

```bash
# Write with automatic erase and verification (recommended)
flash-programmer-tool --port /dev/ttyACM0 write \
  --file firmware.bin --address 0x0 --erase --verify

# Write to specific address
flash-programmer-tool --port /dev/ttyACM0 write \
  --file data.bin --address 0x100000 --erase --verify

# Basic write mode (slower but more reliable)
flash-programmer-tool --port /dev/ttyACM0 write \
  --file data.bin --address 0x0 --erase --verify --basic
```

### 🗑️ Erase Flash Sectors

```bash
# Erase 4KB sector at address 0x0
flash-programmer-tool --port /dev/ttyACM0 erase \
  --address 0x0 --size 0x1000

# Erase entire flash (16MB)
flash-programmer-tool --port /dev/ttyACM0 erase \
  --address 0x0 --size 0x1000000
```

### ✅ Verify Flash Content

```bash
# Verify file against flash content
flash-programmer-tool --port /dev/ttyACM0 verify \
  --file firmware.bin --address 0x0
```

### 📊 Check Flash Status

```bash
flash-programmer-tool --port /dev/ttyACM0 status
```

**Output:**

```text
Flash Status Register: 0x00
  Write In Progress (WIP): No
  Write Enable Latch (WEL): No
  Block Protect Bits (BP0-BP2): 0x0
  Top/Bottom Protect (TB): Bottom
  Sector Protect (SEC): No
  Status Register Protect (SRP0): No
```

## 🎯 Advanced Usage

### Large File Programming

For large files (like the complete 16MB flash image):

```bash
# Program complete flash content generator output
flash-programmer-tool --port /dev/ttyACM0 --timeout 300 write \
  --file w25q128jv_complete.bin --address 0x000000 --erase --verify
```

### Batch Operations

```bash
# Program multiple files sequentially
for file in *.bin; do
    echo "Programming $file..."
    flash-programmer-tool --port /dev/ttyACM0 write \
        --file "$file" --address 0x0 --erase --verify
done
```

### Performance Optimization

```bash
# Use environment variables for optimization
export RUST_LOG=info                    # Enable detailed logging
export USB_TIMEOUT=30000                # Increase USB timeout

# Run with optimized settings
flash-programmer-tool --port /dev/ttyACM0 write \
  --file large_file.bin --address 0x0 --erase --verify
```

## 🔧 Command Line Options

### Global Options

- `--port, -p`: Serial port to connect to (default: `/dev/ttyACM0`)
- `--baud, -b`: Baud rate (ignored for USB CDC, kept for compatibility)
- `--timeout, -t`: Connection timeout in seconds (default: 10)

### Commands

#### `info`

Get flash chip information including JEDEC ID, size, and sector layout.

#### `status`

Read and decode the flash status register.

#### `erase`

- `--address, -a`: Start address (hex format supported)
- `--size, -s`: Size to erase in bytes (hex format supported)

#### `write`

- `--file, -f`: Input file path
- `--address, -a`: Start address (default: 0x0)
- `--erase, -e`: Erase before writing
- `--verify, -v`: Verify after writing using progressive CRC32
- `--basic, -b`: Use basic write mode instead of stream write

#### `read`

- `--file, -f`: Output file path
- `--address, -a`: Start address (default: 0x0)
- `--size, -s`: Size to read in bytes

#### `verify`

- `--file, -f`: File to verify against flash
- `--address, -a`: Start address (default: 0x0)

### Address Format

Addresses can be specified in decimal or hexadecimal:

- Decimal: `1048576`
- Hexadecimal: `0x100000` or `0X100000`

## 📊 Performance Metrics

| Operation | File Size | Time | Speed | Notes |
|-----------|-----------|------|-------|-------|
| Read | 1KB | <1s | Instant | Small file optimization |
| Read | 1MB | ~1.5min | 11.4 KB/s | Bulk transfer |
| Read | 16MB | ~26min | 10.7 KB/s | Full flash |
| Write | 1KB | <1s | Instant | Page-aligned |
| Write | 1MB | ~2min | 8.5 KB/s | With erase+verify |
| Write | 16MB | ~35min | 7.8 KB/s | Complete flash |
| Erase | 4KB | <1s | Instant | Single sector |
| Erase | 16MB | ~8min | N/A | Chip erase |

## 🐛 Troubleshooting

### Connection Issues

```bash
# Check available USB devices
ls /dev/tty* | grep -E "(ACM|USB)"

# Common port names:
# Linux: /dev/ttyACM0, /dev/ttyUSB0
# macOS: /dev/cu.usbmodem*, /dev/cu.usbserial*
# Windows: COM3, COM4, etc.
```

### Permission Issues (Linux)

```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Or use sudo for one-time access
sudo ./target/release/flash-programmer-tool --port /dev/ttyACM0 info
```

### Timeout Issues

```bash
# Increase timeout for large operations
flash-programmer-tool --port /dev/ttyACM0 --timeout 300 write \
  --file large_file.bin --address 0x0 --erase --verify
```

### Verification Failures

```bash
# Use basic write mode for problematic files
flash-programmer-tool --port /dev/ttyACM0 write \
  --file data.bin --address 0x0 --erase --verify --basic

# Verify separately after writing
flash-programmer-tool --port /dev/ttyACM0 verify \
  --file data.bin --address 0x0
```

## 🔗 Integration

This tool is designed to work with:

- **Flash Content Generator**: Program generated flash images
- **STM32G4 Projects**: Read/write application data and resources
- **CI/CD Pipelines**: Automated flash programming
- **Development Workflows**: Rapid prototyping and testing

## 📄 License

This project is part of the STM32G4 Flash Programmer toolkit.
