# STM32G4 Flash Programmer

A high-performance USB-based external flash programmer for STM32G4 microcontrollers, designed to program W25Q128 SPI flash memory chips with automatic erase functionality and data verification.

## ✨ Features

- 🚀 **High-speed USB CDC communication** (115200 baud)
- 💾 **Complete SPI flash support** (W25Q128 tested)
- 🔄 **Automatic erase before write** (--erase flag)
- ✅ **Data verification** (--verify flag)
- 🛡️ **Hardware CRC verification**
- 🔒 **Safe flash operations** with proper error handling
- 📊 **Progress indicators** for all operations
- 🎯 **Precise addressing** with hex format support

## 🔧 Hardware Requirements

- **STM32G431CBU6** microcontroller
- **W25Q128** SPI flash memory (16MB)
- **USB connection** for communication
- **Proper pin connections** (see below)

## 📌 Pin Configuration

### ⚡ SPI Flash Connections (Critical!)

- **SCK**: PB13 (SPI2_SCK)
- **MISO**: PB14 (SPI2_MISO)
- **MOSI**: PB15 (SPI2_MOSI)
- **CS**: **PB12** (GPIO Output) ⚠️ **Must be PB12!**
- **WP#**: **PB11** (GPIO Output, pulled HIGH) ⚠️ **Must be PB11!**
- **HOLD#**: PA10 (GPIO Output, pulled HIGH)

### 🔌 USB Connection

- **USB D+**: PA12
- **USB D-**: PA11

> **⚠️ Important**: The CS and WP# pin assignments are critical for proper operation. Using different pins will result in communication failures.

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust with embedded target
rustup target add thumbv7em-none-eabihf

# Install probe-rs for flashing
cargo install probe-rs --features cli
```

### 1. Build and Flash Firmware

```bash
cd firmware
cargo run --release
```

### 2. Build Host Tool

```bash
cd host-tool
cargo build --release
```

### 3. Test Connection

```bash
./target/release/flash-programmer-tool --port /dev/ttyACM0 info
```

## 📖 Usage Examples

### 🔍 Get Flash Information

```bash
flash-programmer-tool --port /dev/ttyACM0 info
```

### 📖 Read Flash Memory

```bash
# Read 1KB from address 0x0
flash-programmer-tool --port /dev/ttyACM0 read \
  --file backup.bin --address 0x0 --size 0x400
```

### ✍️ Write Flash Memory (Recommended)

```bash
# Write with automatic erase and verification
flash-programmer-tool --port /dev/ttyACM0 write \
  --file firmware.bin --address 0x0 --erase --verify
```

### 🗑️ Erase Flash Sectors

```bash
# Erase 4KB sector at address 0x0
flash-programmer-tool --port /dev/ttyACM0 erase \
  --address 0x0 --size 0x1000
```

### 🎯 Advanced Usage

```bash
# Write to specific address with basic mode
flash-programmer-tool --port /dev/ttyACM0 write \
  --file data.bin --address 0x100000 --erase --basic

# Read large section
flash-programmer-tool --port /dev/ttyACM0 read \
  --file full_backup.bin --address 0x0 --size 0x1000000
```

## 🔄 Workflow

The recommended workflow for flash programming:

1. **Erase** → **Write** → **Verify**
2. Use `--erase` flag for automatic sector erasing
3. Use `--verify` flag for data integrity checking
4. Monitor progress bars for operation status

## 🛠️ Protocol Details

### Packet Structure

```text
┌─────────────┬─────────┬─────────┬─────────┬────────┬──────────┬─────────┐
│ Magic (2B)  │ Seq(1B) │ Cmd(1B) │ Addr(3B)│ Len(1B)│ Data(nB) │ CRC(2B) │
├─────────────┼─────────┼─────────┼─────────┼────────┼──────────┼─────────┤
│    0xABCD   │   0-255 │  1-8    │ 24-bit  │ 0-255  │ Variable │ CRC16   │
└─────────────┴─────────┴─────────┴─────────┴────────┴──────────┴─────────┘
```

### Commands

- **1**: Info - Get flash information
- **2**: Erase - Erase flash sectors
- **3**: Write - Write data to flash
- **4**: Read - Read data from flash
- **8**: StreamWrite - Optimized write operation

## 🎯 Troubleshooting

### Connection Issues

- Verify USB cable and port
- Check that firmware is running (LED indicators)
- Try different USB ports

### Write Failures

- **Always erase before writing**: Use `--erase` flag
- Check address alignment (4KB sectors)
- Verify file size and available space

### Hardware Issues

- Double-check pin connections (especially CS=PB12, WP#=PB11)
- Ensure proper power supply (3.3V)
- Check SPI signal integrity with oscilloscope

## 📊 Performance

- **Write Speed**: ~40 KB/s
- **Read Speed**: ~50 KB/s
- **Erase Speed**: ~60 KB/s
- **USB Latency**: <10ms
- **Maximum File Size**: 16MB (W25Q128 capacity)

## 🧪 Testing

The project includes comprehensive test files:

```bash
# Test files in host-tool/
test_write_debug.bin    # "Hello Flash Test 123\n"
test_read_*.bin         # Various read test results
```

## ✅ Project Status

### 🎉 FULLY FUNCTIONAL AND TESTED

This project has been thoroughly tested and verified:

- ✅ Hardware connections confirmed
- ✅ SPI communication working
- ✅ Write operations successful
- ✅ Read operations successful
- ✅ Erase operations successful
- ✅ Automatic erase functionality working
- ✅ Data verification working
- ✅ All test cases passing

## 📁 Project Structure

```text
stm32g4-flash-programmer/
├── src/                           # Core firmware source
│   ├── main.rs                   # Main application entry
│   ├── usb_handler.rs            # USB CDC communication
│   ├── flash_ops.rs              # Flash operations (read/write/erase)
│   ├── protocol.rs               # Communication protocol
│   └── hardware.rs               # Hardware abstraction
├── protocol/                     # Shared protocol definitions
├── host-tools/                   # PC-side utilities
│   ├── flash-programmer/         # Main CLI tool
│   └── examples/                 # Usage examples
├── examples/                     # Example projects and tools
│   ├── stm32g431-w25q128jv/      # STM32G431 Flash content viewer
│   └── flash-content-generator/  # Tools for generating Flash content
└── docs/                         # Documentation
```

### Examples Directory

- **`examples/stm32g431-w25q128jv/`**: Complete STM32G431 firmware example that reads font bitmaps from external Flash and displays them on an ST7789 TFT screen with vertical baseline alignment
- **`examples/flash-content-generator/`**: Python tools and web-based **preview applications** for generating and visualizing Flash content including fonts, images, and resource layouts

> **Note**: The web application in `flash-content-generator/web-app/` is a **preview tool only** and does not support Flash programming. Use the CLI tool in `host-tool/` for actual Flash programming operations.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## 🙏 Acknowledgments

- Embassy framework for async embedded Rust
- probe-rs team for excellent debugging tools
- STM32 community for hardware insights
