#!/usr/bin/env python3
"""
Verify Flash image contains correct font data
"""

import os
import struct

def verify_flash_font_data():
    """Verify the 'F' character in the flash image"""
    flash_image_path = '../w25q128jv_complete.bin'
    font_file_path = '../assets/font_output/font_bitmap.bin'
    
    if not os.path.exists(flash_image_path):
        print(f"Flash image not found: {flash_image_path}")
        return
    
    if not os.path.exists(font_file_path):
        print(f"Font file not found: {font_file_path}")
        return
    
    print("🔍 Verifying Flash image font data...")
    
    # Read 'F' character from original font file
    with open(font_file_path, 'rb') as f:
        # Read header
        char_count_data = f.read(4)
        char_count = struct.unpack('<I', char_count_data)[0]
        
        # Find 'F' character
        target_char = 0x0046
        f_char_info = None
        
        for i in range(char_count):
            char_info = f.read(8)
            if len(char_info) < 8:
                break
                
            char_code, width, height, bitmap_offset = struct.unpack('<IBBH', char_info)
            
            if char_code == target_char:
                f_char_info = {
                    'char_code': char_code,
                    'width': width,
                    'height': height,
                    'bitmap_offset': bitmap_offset
                }
                
                # Read bitmap data
                current_pos = f.tell()
                f.seek(bitmap_offset)
                bytes_per_row = (width + 7) // 8
                bitmap_size = bytes_per_row * height
                bitmap_data = f.read(bitmap_size)
                f_char_info['bitmap_data'] = bitmap_data
                f.seek(current_pos)
                break
    
    if not f_char_info:
        print("❌ 'F' character not found in font file!")
        return
    
    print(f"✅ Found 'F' in font file:")
    print(f"   Size: {f_char_info['width']}x{f_char_info['height']}")
    print(f"   Bitmap: {list(f_char_info['bitmap_data'])}")
    print(f"   Hex: {' '.join(f'{b:02X}' for b in f_char_info['bitmap_data'])}")
    
    # Now check the same data in flash image
    font_base_address = 0x00020000
    
    with open(flash_image_path, 'rb') as f:
        # Seek to font area
        f.seek(font_base_address)
        
        # Read header
        char_count_data = f.read(4)
        char_count = struct.unpack('<I', char_count_data)[0]
        print(f"\n🔍 Flash image font area:")
        print(f"   Character count: {char_count}")
        
        # Find 'F' character in flash
        flash_f_char_info = None
        
        for i in range(char_count):
            char_info = f.read(8)
            if len(char_info) < 8:
                break
                
            char_code, width, height, bitmap_offset = struct.unpack('<IBBH', char_info)
            
            if char_code == target_char:
                flash_f_char_info = {
                    'char_code': char_code,
                    'width': width,
                    'height': height,
                    'bitmap_offset': bitmap_offset
                }
                
                # Read bitmap data from flash
                current_pos = f.tell()
                f.seek(font_base_address + bitmap_offset)
                bytes_per_row = (width + 7) // 8
                bitmap_size = bytes_per_row * height
                bitmap_data = f.read(bitmap_size)
                flash_f_char_info['bitmap_data'] = bitmap_data
                f.seek(current_pos)
                break
    
    if not flash_f_char_info:
        print("❌ 'F' character not found in flash image!")
        return
    
    print(f"✅ Found 'F' in flash image:")
    print(f"   Size: {flash_f_char_info['width']}x{flash_f_char_info['height']}")
    print(f"   Bitmap: {list(flash_f_char_info['bitmap_data'])}")
    print(f"   Hex: {' '.join(f'{b:02X}' for b in flash_f_char_info['bitmap_data'])}")
    
    # Compare the data
    print(f"\n📊 Comparison:")
    font_data = f_char_info['bitmap_data']
    flash_data = flash_f_char_info['bitmap_data']
    
    if font_data == flash_data:
        print("✅ Font data matches perfectly!")
    else:
        print("❌ Font data MISMATCH!")
        print(f"   Font file:  {list(font_data)}")
        print(f"   Flash image: {list(flash_data)}")
        
        # Show differences
        for i, (f_byte, fl_byte) in enumerate(zip(font_data, flash_data)):
            if f_byte != fl_byte:
                print(f"   Byte {i}: Font=0x{f_byte:02X}, Flash=0x{fl_byte:02X}")

if __name__ == "__main__":
    verify_flash_font_data()
