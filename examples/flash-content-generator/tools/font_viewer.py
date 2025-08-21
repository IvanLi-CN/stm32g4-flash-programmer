#!/usr/bin/env python3
"""
Font Binary File Viewer
Utility to inspect and validate generated font binary files.

This tool can:
1. Display font file structure and metadata
2. Show character information table
3. Render character bitmaps as ASCII art
4. Validate file format integrity

Author: AI Assistant (白羽)
Date: 2025-01-20
"""

import os
import sys
import struct
import argparse
from typing import List, Tuple, Optional


class CharacterInfo:
    """Character information structure"""
    def __init__(self, unicode_val: int, width: int, height: int, bitmap_offset: int):
        self.unicode = unicode_val
        self.width = width
        self.height = height
        self.bitmap_offset = bitmap_offset
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'CharacterInfo':
        """Create CharacterInfo from 10-byte binary data"""
        unicode_val, width, height, bitmap_offset = struct.unpack('<IBBI', data)
        return cls(unicode_val, width, height, bitmap_offset)
    
    def __str__(self) -> str:
        char_repr = chr(self.unicode) if 32 <= self.unicode <= 126 else f"U+{self.unicode:04X}"
        return f"'{char_repr}' (U+{self.unicode:04X}): {self.width}×{self.height} @ 0x{self.bitmap_offset:08X}"


class FontViewer:
    """Font binary file viewer and validator"""
    
    def __init__(self, font_path: str):
        """Initialize with font file path"""
        self.font_path = font_path
        self.characters = []
        self.font_data = None
        
    def load_font(self) -> bool:
        """Load and parse font file"""
        try:
            with open(self.font_path, 'rb') as f:
                self.font_data = f.read()
            
            if len(self.font_data) < 4:
                print(f"❌ Font file too small: {len(self.font_data)} bytes")
                return False
            
            # Read character count
            char_count = struct.unpack('<I', self.font_data[:4])[0]
            print(f"📊 Character count: {char_count}")
            
            if char_count == 0 or char_count > 10000:
                print(f"❌ Invalid character count: {char_count}")
                return False
            
            # Read character info array
            expected_info_size = 4 + char_count * 10
            if len(self.font_data) < expected_info_size:
                print(f"❌ Font file too small for character info: {len(self.font_data)} < {expected_info_size}")
                return False
            
            self.characters = []
            for i in range(char_count):
                offset = 4 + i * 10
                char_data = self.font_data[offset:offset + 10]
                char_info = CharacterInfo.from_bytes(char_data)
                self.characters.append(char_info)
            
            print(f"✅ Loaded {len(self.characters)} characters")
            return True
            
        except Exception as e:
            print(f"❌ Failed to load font: {e}")
            return False
    
    def show_info(self):
        """Display font file information"""
        if not self.font_data:
            print("❌ No font data loaded")
            return
        
        print(f"\n📁 Font File: {self.font_path}")
        print(f"📏 File Size: {len(self.font_data)} bytes")
        print(f"🔢 Character Count: {len(self.characters)}")
        
        if self.characters:
            # Analyze character dimensions
            widths = [char.width for char in self.characters]
            heights = [char.height for char in self.characters]
            
            print(f"📐 Character Dimensions:")
            print(f"   Width: {min(widths)}-{max(widths)} (avg: {sum(widths)/len(widths):.1f})")
            print(f"   Height: {min(heights)}-{max(heights)} (avg: {sum(heights)/len(heights):.1f})")
            
            # Check if monospace
            if len(set(widths)) == 1 and len(set(heights)) == 1:
                print(f"✅ Monospace font: {widths[0]}×{heights[0]}")
            else:
                print(f"⚠️ Variable width font")
            
            # Unicode range
            unicodes = [char.unicode for char in self.characters]
            print(f"🔤 Unicode Range: U+{min(unicodes):04X} - U+{max(unicodes):04X}")
    
    def show_character_table(self, limit: int = 20):
        """Display character information table"""
        if not self.characters:
            print("❌ No characters loaded")
            return
        
        print(f"\n📋 Character Table (showing first {min(limit, len(self.characters))}):")
        print("─" * 60)
        
        for i, char in enumerate(self.characters[:limit]):
            print(f"{i:3d}: {char}")
        
        if len(self.characters) > limit:
            print(f"... and {len(self.characters) - limit} more characters")
    
    def render_character_ascii(self, char_index: int) -> bool:
        """Render character bitmap as ASCII art"""
        if char_index >= len(self.characters):
            print(f"❌ Invalid character index: {char_index}")
            return False
        
        char_info = self.characters[char_index]
        
        # Calculate bitmap size (byte-aligned rows)
        bytes_per_row = (char_info.width + 7) // 8
        bitmap_size = bytes_per_row * char_info.height
        bitmap_start = char_info.bitmap_offset
        bitmap_end = bitmap_start + bitmap_size
        
        if bitmap_end > len(self.font_data):
            print(f"❌ Bitmap data out of bounds: {bitmap_end} > {len(self.font_data)}")
            return False
        
        # Extract bitmap data
        bitmap_data = self.font_data[bitmap_start:bitmap_end]
        
        print(f"\n🎨 Character: {char_info}")
        print(f"📊 Bitmap Size: {bitmap_size} bytes")
        print("─" * (char_info.width + 2))
        
        # Render bitmap (byte-aligned rows)
        bytes_per_row = (char_info.width + 7) // 8
        for y in range(char_info.height):
            line = "|"
            row_start = y * bytes_per_row

            for x in range(char_info.width):
                byte_index = row_start + (x // 8)
                bit_index = 7 - (x % 8)

                if byte_index < len(bitmap_data):
                    pixel = (bitmap_data[byte_index] >> bit_index) & 1
                    line += "█" if pixel else " "
                else:
                    line += "?"
            line += "|"
            print(line)
        
        print("─" * (char_info.width + 2))
        return True
    
    def validate_font(self) -> bool:
        """Validate font file integrity"""
        if not self.font_data or not self.characters:
            print("❌ No font data to validate")
            return False
        
        print("\n🔍 Validating font file...")
        
        errors = 0
        
        # Check character info consistency
        for i, char in enumerate(self.characters):
            # Check dimensions
            if char.width == 0 or char.height == 0:
                print(f"❌ Character {i}: Invalid dimensions {char.width}×{char.height}")
                errors += 1
            
            if char.width > 64 or char.height > 64:
                print(f"⚠️ Character {i}: Large dimensions {char.width}×{char.height}")
            
            # Check bitmap offset
            bitmap_size = (char.width * char.height + 7) // 8
            if char.bitmap_offset + bitmap_size > len(self.font_data):
                print(f"❌ Character {i}: Bitmap out of bounds")
                errors += 1
        
        # Check for duplicate Unicode values
        unicodes = [char.unicode for char in self.characters]
        if len(set(unicodes)) != len(unicodes):
            print(f"❌ Duplicate Unicode values found")
            errors += 1
        
        # Check Unicode order (should be sorted)
        if unicodes != sorted(unicodes):
            print(f"⚠️ Characters not sorted by Unicode value")
        
        if errors == 0:
            print("✅ Font file validation passed")
            return True
        else:
            print(f"❌ Font file validation failed with {errors} errors")
            return False


def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="View and validate font binary files")
    parser.add_argument("font_file", help="Path to font binary file")
    parser.add_argument("--info", "-i", action="store_true", 
                       help="Show font information")
    parser.add_argument("--table", "-t", type=int, default=0, metavar="N",
                       help="Show character table (limit N entries)")
    parser.add_argument("--render", "-r", type=int, metavar="INDEX",
                       help="Render character at index as ASCII art")
    parser.add_argument("--validate", "-v", action="store_true",
                       help="Validate font file integrity")
    parser.add_argument("--all", "-a", action="store_true",
                       help="Show all information")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.font_file):
        print(f"❌ Font file not found: {args.font_file}")
        sys.exit(1)
    
    # Load font
    viewer = FontViewer(args.font_file)
    if not viewer.load_font():
        sys.exit(1)
    
    # Show information based on arguments
    if args.all or args.info:
        viewer.show_info()
    
    if args.all or args.table > 0:
        limit = args.table if args.table > 0 else 20
        viewer.show_character_table(limit)
    
    if args.render is not None:
        viewer.render_character_ascii(args.render)
    
    if args.all or args.validate:
        viewer.validate_font()
    
    # Default: show basic info if no specific options
    if not any([args.info, args.table, args.render is not None, args.validate, args.all]):
        viewer.show_info()


if __name__ == "__main__":
    main()
