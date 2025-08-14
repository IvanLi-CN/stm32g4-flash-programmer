#!/usr/bin/env python3
"""
Font Bitmap Converter for STM32G4 Flash Programmer
Converts TTF fonts to bitmap format for embedded systems
"""

import os
import sys
import struct
from PIL import Image, ImageDraw, ImageFont

def convert_font_to_bitmap(font_path, output_dir, font_size=12):
    """Convert TTF font to bitmap format for embedded systems"""
    print(f"Converting font: {font_path}")
    print(f"Font size: {font_size}px")

    try:
        # Load font
        font = ImageFont.truetype(font_path, font_size)

        # Create output directory
        os.makedirs(output_dir, exist_ok=True)

        # Define character ranges to convert
        char_ranges = [
            (0x20, 0x7E, "ASCII"),           # Basic ASCII
            (0x4E00, 0x9FFF, "CJK_UNIFIED"), # CJK Unified Ideographs (common Chinese)
            (0x3400, 0x4DBF, "CJK_EXT_A"),   # CJK Extension A
        ]

        all_char_data = []
        total_chars = 0

        for start, end, name in char_ranges:
            print(f"Processing {name} range: 0x{start:04X} - 0x{end:04X}")
            char_data = extract_character_range(font, start, end, name)
            all_char_data.extend(char_data)
            total_chars += len(char_data)
            print(f"  Extracted {len(char_data)} characters")

        # Sort characters by Unicode code point for binary search optimization
        print("Sorting characters by Unicode code point...")
        all_char_data.sort(key=lambda x: x['char_code'])
        print(f"✓ Sorted {len(all_char_data)} characters")

        # Generate bitmap file
        bitmap_path = os.path.join(output_dir, 'font_bitmap.bin')
        generate_bitmap_file(all_char_data, bitmap_path)

        # Generate info file
        info_path = os.path.join(output_dir, 'font_info.txt')
        generate_bitmap_info(all_char_data, info_path, font_size, total_chars)

        print(f"\n✓ Font conversion complete!")
        print(f"Total characters: {total_chars}")
        print(f"Bitmap file: {bitmap_path}")
        print(f"Info file: {info_path}")

        return True

    except Exception as e:
        print(f"Error converting font: {e}")
        return False

def extract_character_range(font, start_code, end_code, range_name):
    """Extract character bitmaps from a Unicode range"""
    char_data = []

    for char_code in range(start_code, min(end_code + 1, start_code + 1000)):  # Limit to prevent huge files
        try:
            char = chr(char_code)

            # Get character dimensions
            bbox = font.getbbox(char)
            if bbox[2] <= bbox[0] or bbox[3] <= bbox[1]:
                continue  # Skip characters with no size

            char_width = bbox[2] - bbox[0]
            char_height = bbox[3] - bbox[1]

            # Limit character size to reasonable bounds
            if char_width > 32 or char_height > 32:
                continue

            # Create image for character
            img = Image.new('L', (char_width, char_height), 0)
            draw = ImageDraw.Draw(img)

            # Draw character
            draw.text((-bbox[0], -bbox[1]), char, font=font, fill=255)

            # Convert to 1-bit bitmap
            bitmap_data = convert_to_bitmap(img)

            char_info = {
                'char_code': char_code,
                'char': char,
                'width': char_width,
                'height': char_height,
                'bitmap_size': len(bitmap_data),
                'bitmap_data': bitmap_data,
                'range': range_name
            }

            char_data.append(char_info)

        except (ValueError, OSError):
            # Skip characters that can't be rendered
            continue

    return char_data

def convert_to_bitmap(img):
    """Convert PIL image to 1-bit bitmap data"""
    width, height = img.size
    bitmap_data = bytearray()
    pixels = img.load()

    for y in range(height):
        byte_val = 0
        bit_count = 0

        for x in range(width):
            # Convert grayscale to 1-bit (threshold at 128)
            pixel = 1 if pixels[x, y] > 128 else 0
            byte_val |= (pixel << (7 - bit_count))
            bit_count += 1

            if bit_count == 8:
                bitmap_data.append(byte_val)
                byte_val = 0
                bit_count = 0

        # Handle remaining bits in the row
        if bit_count > 0:
            bitmap_data.append(byte_val)

    return bytes(bitmap_data)

def generate_bitmap_file(char_data, output_path):
    """Generate binary bitmap file"""
    with open(output_path, 'wb') as f:
        # Write header
        f.write(struct.pack('<I', len(char_data)))  # Number of characters

        # Calculate bitmap offsets
        header_size = 4  # Character count
        char_info_size = len(char_data) * 8  # 8 bytes per character info
        bitmap_offset = header_size + char_info_size

        # Write character info table
        for char_info in char_data:
            f.write(struct.pack('<I', char_info['char_code']))  # Unicode code point
            f.write(struct.pack('<B', char_info['width']))      # Width
            f.write(struct.pack('<B', char_info['height']))     # Height
            f.write(struct.pack('<H', bitmap_offset))           # Bitmap offset
            bitmap_offset += char_info['bitmap_size']

        # Write bitmap data
        for char_info in char_data:
            f.write(char_info['bitmap_data'])

def generate_bitmap_info(char_data, output_path, font_size, total_chars):
    """Generate text file with bitmap font information"""
    with open(output_path, 'w') as f:
        f.write("WenQuanYi Bitmap Font Information\n")
        f.write("=================================\n\n")

        # Font info
        f.write(f"Font Size: {font_size}px\n")
        f.write(f"Total Characters: {total_chars}\n")
        f.write(f"Format: 1-bit monochrome bitmap\n")
        f.write(f"Encoding: Unicode (UTF-16)\n\n")

        # Character ranges
        ranges = {}
        for char_info in char_data:
            range_name = char_info['range']
            if range_name not in ranges:
                ranges[range_name] = []
            ranges[range_name].append(char_info)

        f.write("Character Ranges:\n")
        f.write("=================\n")
        for range_name, chars in ranges.items():
            f.write(f"{range_name}: {len(chars)} characters\n")
            if chars:
                min_code = min(c['char_code'] for c in chars)
                max_code = max(c['char_code'] for c in chars)
                f.write(f"  Range: U+{min_code:04X} - U+{max_code:04X}\n")
        f.write("\n")

        # Binary format
        f.write("Binary Format:\n")
        f.write("==============\n")
        f.write("Header (4 bytes):\n")
        f.write("  - Character count (uint32_t, little-endian)\n\n")
        f.write("Character Info Table (8 bytes per character):\n")
        f.write("  - Unicode code point (uint32_t, little-endian)\n")
        f.write("  - Width (uint8_t)\n")
        f.write("  - Height (uint8_t)\n")
        f.write("  - Bitmap offset (uint16_t, little-endian)\n\n")
        f.write("Bitmap Data:\n")
        f.write("  - 1-bit monochrome bitmap\n")
        f.write("  - Packed 8 pixels per byte\n")
        f.write("  - Row-major order\n\n")

        # Verify sorting
        f.write("Sorting Verification:\n")
        f.write("====================\n")
        is_sorted = all(char_data[i]['char_code'] <= char_data[i+1]['char_code']
                       for i in range(len(char_data)-1))
        f.write(f"Characters sorted: {'✓ YES' if is_sorted else '✗ NO'}\n")
        if len(char_data) > 0:
            f.write(f"First character: U+{char_data[0]['char_code']:04X} ('{char_data[0]['char']}')\n")
            f.write(f"Last character: U+{char_data[-1]['char_code']:04X} ('{char_data[-1]['char']}')\n")
        f.write(f"Binary search ready: {'✓ YES' if is_sorted else '✗ NO'}\n\n")

        # Sample characters (sorted)
        f.write("Sample Characters (Sorted Order):\n")
        f.write("=================================\n")
        f.write("Char | Unicode | Size  | Bitmap Size\n")
        f.write("-----|---------|-------|------------\n")
        for i, char_info in enumerate(char_data[:20]):  # Show first 20
            f.write(f" '{char_info['char']}'  | U+{char_info['char_code']:04X}  | "
                   f"{char_info['width']:2d}x{char_info['height']:2d} | "
                   f"{char_info['bitmap_size']:3d} bytes\n")
        if len(char_data) > 20:
            f.write(f"... and {len(char_data) - 20} more characters\n")

def main():
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(os.path.dirname(script_dir), 'assets')

    # Font paths
    font_path = os.path.join(assets_dir, 'WenQuanYi.Bitmap.Song.12px.ttf')
    output_dir = os.path.join(assets_dir, 'font_output')

    print("=== Font Bitmap Converter ===")
    print(f"Input font: {font_path}")
    print(f"Output directory: {output_dir}")

    # Check if font exists
    if not os.path.exists(font_path):
        print(f"Error: Font file not found: {font_path}")
        return 1

    # Convert font to bitmap
    if convert_font_to_bitmap(font_path, output_dir, font_size=12):
        print("\n=== Font Conversion Complete ===")
        print("Bitmap font is ready for flash programming.")
        print("Use the address from memory_map.txt for programming.")
        return 0
    else:
        return 1

if __name__ == "__main__":
    exit(main())
