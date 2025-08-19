#!/usr/bin/env python3
"""
Analyze font bitmap data to understand the 'F' character issue
"""

import os
import struct

def analyze_font_file():
    """Analyze the generated font bitmap file"""
    font_path = '../assets/font_output/font_bitmap_12px.bin'
    
    if not os.path.exists(font_path):
        print(f"Font file not found: {font_path}")
        return
    
    with open(font_path, 'rb') as f:
        # Read header
        char_count_data = f.read(4)
        char_count = struct.unpack('<I', char_count_data)[0]
        print(f"Font contains {char_count} characters")
        
        # Find ASCII characters and analyze range
        ascii_chars = []
        first_10_chars = []

        for i in range(min(char_count, 100)):  # Check first 100 characters
            # Read character info (10 bytes: 4+1+1+4)
            char_info = f.read(10)
            if len(char_info) < 10:
                break

            char_code, width, height, bitmap_offset = struct.unpack('<IBBI', char_info)
            if i < 5:  # Only show debug for first 5 chars
                print(f"   DEBUG: Raw bytes: {char_info.hex()}")
                print(f"   DEBUG: Parsed - char_code=0x{char_code:X}, width={width}, height={height}, offset=0x{bitmap_offset:X}")

            if i < 10:
                first_10_chars.append((char_code, chr(char_code) if 32 <= char_code <= 126 else '?'))

            if 0x20 <= char_code <= 0x7E:  # ASCII printable range
                ascii_chars.append((char_code, chr(char_code)))

        print(f"\n📊 First 10 characters in font:")
        for char_code, char_display in first_10_chars:
            print(f"   U+{char_code:04X} = '{char_display}'")

        print(f"\n📊 ASCII characters found (U+0020-U+007E):")
        if ascii_chars:
            for char_code, char_display in ascii_chars[:20]:  # Show first 20 ASCII chars
                print(f"   U+{char_code:04X} = '{char_display}'")
            if len(ascii_chars) > 20:
                print(f"   ... and {len(ascii_chars) - 20} more ASCII characters")
        else:
            print("   ❌ No ASCII characters found!")

        # Reset file position and look for specific characters
        f.seek(4)  # Back to start of character data
        target_chars = [0x0021, 0x0041, 0x0046, 0x0048]  # !, A, F, H
        found_chars = {}

        for i in range(char_count):
            char_info = f.read(10)
            if len(char_info) < 10:
                break

            char_code, width, height, bitmap_offset = struct.unpack('<IBBI', char_info)

            if char_code in target_chars:
                found_chars[char_code] = {
                    'width': width,
                    'height': height,
                    'bitmap_offset': bitmap_offset
                }

        print(f"\n🎯 Target character analysis:")
        for target_char in target_chars:
            char_name = {0x0021: '!', 0x0041: 'A', 0x0046: 'F', 0x0048: 'H'}[target_char]
            if target_char in found_chars:
                info = found_chars[target_char]
                print(f"   ✅ '{char_name}' (U+{target_char:04X}): {info['width']}x{info['height']} at offset 0x{info['bitmap_offset']:04X}")
            else:
                print(f"   ❌ '{char_name}' (U+{target_char:04X}): NOT FOUND")

        # Show character range analysis
        f.seek(4)  # Back to start
        min_char = float('inf')
        max_char = 0

        for i in range(char_count):
            char_info = f.read(10)
            if len(char_info) < 10:
                break
            char_code = struct.unpack('<I', char_info[:4])[0]
            min_char = min(min_char, char_code)
            max_char = max(max_char, char_code)

        print(f"\n📊 Character range in font:")
        print(f"   Minimum: U+{min_char:04X} ({chr(min_char) if 32 <= min_char <= 126 else '?'})")
        print(f"   Maximum: U+{max_char:04X} ({chr(max_char) if 32 <= max_char <= 126 else '?'})")
        print(f"   Total characters: {char_count}")

if __name__ == "__main__":
    analyze_font_file()
