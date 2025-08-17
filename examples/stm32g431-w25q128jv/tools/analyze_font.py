#!/usr/bin/env python3
"""
Analyze font bitmap data to understand the 'F' character issue
"""

import os
import struct

def analyze_font_file():
    """Analyze the generated font bitmap file"""
    font_path = '../assets/font_output/font_bitmap.bin'
    
    if not os.path.exists(font_path):
        print(f"Font file not found: {font_path}")
        return
    
    with open(font_path, 'rb') as f:
        # Read header
        char_count_data = f.read(4)
        char_count = struct.unpack('<I', char_count_data)[0]
        print(f"Font contains {char_count} characters")
        
        # Find 'F' character (U+0046)
        target_char = 0x0046
        found = False
        
        for i in range(char_count):
            # Read character info (8 bytes)
            char_info = f.read(8)
            if len(char_info) < 8:
                break
                
            char_code, width, height, bitmap_offset = struct.unpack('<IBBH', char_info)
            
            if char_code == target_char:
                print(f"\n🎯 Found 'F' character (U+{char_code:04X}):")
                print(f"   Width: {width} pixels")
                print(f"   Height: {height} pixels") 
                print(f"   Bitmap offset: 0x{bitmap_offset:04X}")
                
                # Calculate bitmap size
                # Each row is packed, so we need ceil(width/8) bytes per row
                bytes_per_row = (width + 7) // 8
                expected_bitmap_size = bytes_per_row * height
                print(f"   Expected bitmap size: {expected_bitmap_size} bytes ({bytes_per_row} bytes/row)")
                
                # Read bitmap data
                current_pos = f.tell()
                f.seek(bitmap_offset)
                bitmap_data = f.read(expected_bitmap_size)
                
                print(f"   Actual bitmap data: {list(bitmap_data)}")
                print(f"   Hex: {' '.join(f'{b:02X}' for b in bitmap_data)}")
                
                # Analyze bit patterns
                print(f"\n📊 Bit pattern analysis:")
                for row in range(height):
                    if row < len(bitmap_data):
                        byte_val = bitmap_data[row]
                        binary = f"{byte_val:08b}"
                        # Show only the relevant bits for this character width
                        relevant_bits = binary[:width] if width <= 8 else binary
                        print(f"   Row {row}: 0x{byte_val:02X} = {binary} -> '{relevant_bits}' (width={width})")
                
                # Try to visualize the character
                print(f"\n🎨 Character visualization (MSB first):")
                for row in range(height):
                    if row < len(bitmap_data):
                        byte_val = bitmap_data[row]
                        line = ""
                        for bit in range(width):
                            if bit < 8:
                                pixel = (byte_val >> (7 - bit)) & 1
                                line += "█" if pixel else "·"
                            else:
                                line += "?"
                        print(f"   {line}")
                
                found = True
                f.seek(current_pos)
                break
        
        if not found:
            print(f"❌ Character 'F' (U+{target_char:04X}) not found in font!")

if __name__ == "__main__":
    analyze_font_file()
