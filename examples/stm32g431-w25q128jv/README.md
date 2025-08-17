# STM32G431 Flash Content Viewer

STM32G431CBU6-based W25Q128JV Flash content viewer firmware that can display fonts and image resources stored in external Flash on an ST7789 TFT display.

## 🎯 Features

- **Flash Font Rendering**: Read font bitmaps from external Flash and display text on screen
- **Vertical Baseline Alignment**: All characters aligned on the same horizontal baseline
- **Dual SPI Bus Management**: SPI1 connects to ST7789 display, SPI2 connects to W25Q128JV Flash
- **Interactive Interface**: Graphical interface controlled by buttons
- **Flash Communication Verification**: Display Flash chip information and connection status

## 🔌 Hardware Configuration

### STM32G431CBU6 Pin Assignment

| Function | Pin | SPI Bus | Purpose |
|----------|-----|---------|---------|
| **W25Q128JV Flash** | | **SPI2** | |
| SCK | PB13 | SPI2 | Clock |
| MOSI | PB15 | SPI2 | Master Out Slave In |
| MISO | PB14 | SPI2 | Master In Slave Out |
| CS | PB12 | - | Chip Select |
| **ST7789 Display** | | **SPI1** | |
| SCK | PB3 | SPI1 | Clock |
| MOSI | PB5 | SPI1 | Data |
| CS | PA15 | - | Chip Select |
| DC | PC14 | - | Data/Command |
| RST | PC15 | - | Reset |
| **User Interaction** | | | |
| Button1 | PC10 | - | Function Button 1 |
| Button3 | PC13 | - | Function Button 3 |

## 🚀 Build and Usage

### Prerequisites

1. **Flash Content Preparation**: Use tools in `../flash-content-generator/` to generate font bitmap files
2. **Flash Programming**: Program the generated font files to W25Q128JV Flash at address 0x20000

### Build Firmware

```bash
# Build Release version
cargo build --release

# Flash to STM32
cargo flash --release --chip STM32G431CBUx
```

### Runtime Effects

1. **Startup**: Firmware displays Flash chip information after startup
2. **Font Testing**: Display font characters read from Flash
3. **Button Labels**: Display "BTN1" and "BTN3" labels on screen
4. **Baseline Alignment**: All text aligned on the same horizontal line

## 📁 Project Structure

```text
src/
├── main.rs              # Main program entry and application logic
├── hardware/            # Hardware abstraction layer
│   ├── flash.rs         # Flash manager
│   └── display.rs       # Display manager
├── resources/           # Resource management system
│   ├── layout.rs        # Memory layout definitions
│   ├── font_parser.rs   # Font parser
│   ├── image_parser.rs  # Image parser
│   └── cache.rs         # Cache system
└── ui/                  # User interface components
    └── app.rs           # Application framework
```

## 🔧 Technical Features

- **Async Architecture**: Asynchronous programming based on Embassy framework
- **Flash Font Rendering**: Read font bitmaps from external Flash and render to screen
- **Vertical Baseline Alignment**: Implement character vertical baseline alignment for neat text display
- **Modular Design**: Clear hardware abstraction and resource management layering
- **Error Handling**: Comprehensive error handling and status feedback

## 📝 Flash Content Generation

Flash content (font bitmaps, etc.) needs to be generated using tools in `../flash-content-generator/`.

For detailed instructions, see: [Flash Content Generator](../flash-content-generator/README.md)

## License

This project is licensed under the MIT License.
