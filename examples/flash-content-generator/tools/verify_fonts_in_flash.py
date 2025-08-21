#!/usr/bin/env python3
"""
Font Verification Tool for Flash Image
Verifies that custom fonts are correctly embedded in the flash image.

Author: AI Assistant (白羽)
Date: 2025-01-20
"""

import os
import sys
import struct
import argparse


def verify_font_at_address(flash_data, address, expected_char_count, font_name):
    """Verify font data at specific address in flash image"""
    print(f"🔍 Verifying {font_name} at address 0x{address:08X}...")
    
    if address + 4 > len(flash_data):
        print(f"❌ Address out of bounds")
        return False
    
    # Read character count (first 4 bytes)
    char_count = struct.unpack('<I', flash_data[address:address+4])[0]
    
    if char_count != expected_char_count:
        print(f"❌ Character count mismatch: expected {expected_char_count}, got {char_count}")
        return False
    
    print(f"✅ Character count: {char_count}")
    
    # Verify character info array
    char_info_start = address + 4
    char_info_size = char_count * 10  # 10 bytes per character
    
    if char_info_start + char_info_size > len(flash_data):
        print(f"❌ Character info array out of bounds")
        return False
    
    print(f"✅ Character info array: {char_info_size} bytes")
    
    # Read first character info as sample
    if char_count > 0:
        first_char_data = flash_data[char_info_start:char_info_start+10]
        unicode_val, width, height, bitmap_offset = struct.unpack('<IBBI', first_char_data)
        
        print(f"✅ First character: U+{unicode_val:04X} ({chr(unicode_val) if 32 <= unicode_val <= 126 else '?'})")
        print(f"   Dimensions: {width}×{height}")
        print(f"   Bitmap offset: 0x{bitmap_offset:08X}")
        
        # Verify bitmap offset is reasonable (relative to font start)
        expected_min_offset = 4 + char_count * 10  # Header + char info array
        if bitmap_offset < expected_min_offset:
            print(f"❌ Invalid bitmap offset: {bitmap_offset} < {expected_min_offset}")
            return False
    
    print(f"✅ {font_name} verification passed")
    return True


def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="Verify fonts in flash image")
    parser.add_argument("flash_file", help="Path to flash image file")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.flash_file):
        print(f"❌ Flash file not found: {args.flash_file}")
        sys.exit(1)
    
    print(f"🔍 Verifying fonts in flash image: {args.flash_file}")
    print("=" * 60)
    
    # Load flash image
    with open(args.flash_file, 'rb') as f:
        flash_data = f.read()
    
    print(f"📁 Flash image size: {len(flash_data):,} bytes ({len(flash_data) // (1024*1024)} MB)")
    print()
    
    # Font addresses from resource_layout.json
    fonts_to_verify = [
        {
            'name': '24×48 Digit Font',
            'address': 0x7D0000,  # 8192000
            'expected_chars': 12
        },
        {
            'name': '16×24 ASCII Font', 
            'address': 0x7D1000,  # 8196096
            'expected_chars': 95
        }
    ]
    
    success_count = 0
    
    for font_info in fonts_to_verify:
        if verify_font_at_address(
            flash_data, 
            font_info['address'], 
            font_info['expected_chars'], 
            font_info['name']
        ):
            success_count += 1
        print()
    
    print("=" * 60)
    print(f"📊 Verification Summary:")
    print(f"   Fonts verified: {success_count}/{len(fonts_to_verify)}")
    
    if success_count == len(fonts_to_verify):
        print("🎉 All fonts verified successfully!")
        print("✅ Custom fonts are correctly embedded in the flash image")
        return 0
    else:
        print("❌ Some font verifications failed!")
        return 1


if __name__ == "__main__":
    exit(main())
