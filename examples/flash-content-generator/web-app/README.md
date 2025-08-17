# Flash Bitmap Font Analyzer

## 🎯 Project Overview

A bitmap font data visualization and analysis tool specifically designed for STM32G4 W25Q128JV Flash chips. It can parse font bitmap data extracted from Flash memory and provides an intuitive visual interface to view and verify character bitmaps.

## ✨ Key Features

### 📁 File Processing
- **Drag & Drop Upload**: Support dragging .bin files to the upload area
- **Format Validation**: Automatic validation of font file format and data integrity
- **Large File Support**: Efficient handling of 2MB+ font data files

### 🔍 Character Analysis
- **Complete Parsing**: Parse font file header, character info table, and bitmap data
- **Character Statistics**: Display total character count, file size, and basic information
- **Format Recognition**: Automatic recognition of 1-bit monochrome bitmap format

### 🎨 Visual Display
- **Grid View**: Display all character thumbnails in grid format
- **Category Browsing**: Browse by digits, uppercase letters, lowercase letters, Chinese characters, symbols
- **Search Function**: Search by character or Unicode code point
- **Detail View**: Click characters to view large images and detailed information

### 🔧 Interactive Features
- **Multi-level Zoom**: Support 1x, 4x, 8x, 16x zoom display
- **Pixel-level Rendering**: Precise pixel rendering using Canvas API
- **Export Function**: Export individual characters as PNG images
- **Responsive Design**: Support for desktop and mobile devices

## 🚀 Usage Instructions

### 1. Prepare Data File
Extract font data from W25Q128JV chip using STM32G4 Flash programmer:
```bash
./flash-programmer-tool --port /dev/ttyUSB0 read \
  --file font_bitmap_extracted.bin \
  --address 0x20000 --size 0x200000
```

### 2. Open Web Application
Open the `index.html` file in your browser

### 3. Upload Font File
- Click the upload area to select a file, or
- Directly drag and drop .bin files to the upload area

### 4. Browse and Analyze
- Use category buttons on the left to filter character types
- Enter characters or Unicode code points in the search box
- Click characters to view detailed information and large images
- Use zoom controls to adjust display size
- Click export button to save character images

## 📊 Supported Character Types

### 🔢 Digits (0-9)
- Unicode range: U+0030 - U+0039
- Contains all Arabic numerals

### 🔠 Uppercase Letters (A-Z)
- Unicode range: U+0041 - U+005A
- Contains all English uppercase letters

### 🔡 Lowercase Letters (a-z)
- Unicode range: U+0061 - U+007A
- Contains all English lowercase letters

### 🀄 Chinese Characters
- Unicode range: U+4E00 - U+9FFF
- Contains common Chinese characters and extensions

### 🔣 Symbols
- Various punctuation marks and special characters
- ASCII symbol range

## 🛠️ Technical Implementation

### Frontend Technology Stack
- **HTML5**: Semantic structure and Canvas API
- **CSS3**: Modern styling and responsive layout
- **JavaScript ES6+**: Native JavaScript with no external dependencies

### Core Algorithms
- **Binary Parsing**: Little-endian data reading
- **Bitmap Rendering**: 1-bit bitmap to pixel conversion
- **Memory Optimization**: Pagination and virtual scrolling

### Data Format Support
```
Font File Structure:
+------------------+
| Header (4 bytes) | Character count (uint32_t, little-endian)
+------------------+
| Character Info   | 8 bytes per character:
| (8 * N bytes)    | - Unicode code point (uint32_t)
|                  | - Width (uint8_t)
|                  | - Height (uint8_t)
|                  | - Bitmap offset (uint16_t)
+------------------+
| Bitmap Data      | 1-bit monochrome bitmap data
| (variable)       | - 8 pixels per byte
|                  | - Row-major order
+------------------+
```

## 🔧 Browser Compatibility

### Recommended Browsers
- Chrome 80+
- Firefox 75+
- Safari 13+
- Edge 80+

### Required Features
- FileReader API
- Canvas 2D Context
- ES6 Classes
- Async/Await

## 📝 Usage Examples

### Verify Digit Characters
1. Upload font file
2. Click "🔢 Digits (0-9)" category
3. Verify that 0-9 characters display correctly

### Check Letter Characters
1. Select "🔠 Uppercase Letters (A-Z)" or "🔡 Lowercase Letters (a-z)"
2. Check letter shapes one by one
3. Use detail view to examine pixel-level details

### Search Specific Characters
1. Enter a character in the search box, e.g., "A"
2. Or enter Unicode code point, e.g., "65" or "0x41"
3. View search results

## 🐛 Troubleshooting

### File Loading Failed
- Check if file format is .bin
- Confirm file size is reasonable (usually around 2MB)
- Verify file is not corrupted

### Character Display Issues
- Check if font file is complete
- Confirm data format meets expectations
- Check browser console for error messages

### Performance Issues
- Large file loading may take time
- Use category functions to reduce displayed character count
- Close other browser tabs to free memory

## 📄 License

MIT License - see LICENSE file for details

## 🤝 Contributing

Issues and Pull Requests are welcome to improve this tool!

## 📞 Support

For questions or suggestions, please create a GitHub Issue or contact the developer.
