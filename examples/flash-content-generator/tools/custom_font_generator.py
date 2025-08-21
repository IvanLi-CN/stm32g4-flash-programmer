#!/usr/bin/env python3
"""
Custom Font Generator for STM32G431CBU6 Project
Generates monospace bitmap fonts for embedded display systems.

This tool creates two specialized fonts:
1. Digital Font (24x48): Numbers 0-9, minus sign (-), decimal point (.)
2. ASCII Font (16x24): Complete printable ASCII character set (32-126)

Output format is compatible with existing font system:
- 4-byte header: character count (little-endian)
- N×10-byte character info: Unicode + width + height + bitmap offset
- Variable-length bitmap data

Author: AI Assistant (白羽)
Date: 2025-01-20
"""

import os
import sys
import struct
from PIL import Image, ImageDraw, ImageFont
from typing import List, Tuple, Dict, Optional
import argparse


class CharacterInfo:
    """Character information structure (10 bytes total)"""
    def __init__(self, unicode_val: int, width: int, height: int, bitmap_offset: int):
        self.unicode = unicode_val
        self.width = width
        self.height = height
        self.bitmap_offset = bitmap_offset
    
    def to_bytes(self) -> bytes:
        """Convert to 10-byte binary format (little-endian)"""
        return struct.pack('<IBBI',
                          self.unicode,      # 4 bytes
                          self.width,        # 1 byte
                          self.height,       # 1 byte
                          self.bitmap_offset # 4 bytes
                          )


class CustomFontGenerator:
    """Custom font generator for embedded systems"""
    
    def __init__(self, font_name: str = "Consolas", fallback_fonts: List[str] = None):
        """
        Initialize font generator
        
        Args:
            font_name: Primary font name to use
            fallback_fonts: List of fallback font names
        """
        self.font_name = font_name
        self.fallback_fonts = fallback_fonts or ["Courier New", "DejaVu Sans Mono", "Liberation Mono"]
        
    def _find_system_font(self, size: int) -> Optional[ImageFont.FreeTypeFont]:
        """Find and load a suitable monospace font"""
        font_candidates = [self.font_name] + self.fallback_fonts
        
        for font_name in font_candidates:
            try:
                # Try to load the font
                font = ImageFont.truetype(font_name, size)
                print(f"✅ Using font: {font_name} (size {size})")
                return font
            except (OSError, IOError):
                continue
        
        # Fallback to default font
        try:
            font = ImageFont.load_default()
            print(f"⚠️ Using default font (size {size})")
            return font
        except Exception as e:
            print(f"❌ Failed to load any font: {e}")
            return None
    
    def _render_character(self, char: str, font: ImageFont.FreeTypeFont, 
                         target_width: int, target_height: int) -> bytes:
        """
        Render a single character to monochrome bitmap
        
        Args:
            char: Character to render
            font: Font to use
            target_width: Target character width
            target_height: Target character height
            
        Returns:
            Bitmap data as bytes (1 bit per pixel, packed)
        """
        # Create image with some padding for better rendering
        padding = 4
        img_width = target_width + padding * 2
        img_height = target_height + padding * 2
        
        # Create white background image
        img = Image.new('RGB', (img_width, img_height), 'white')
        draw = ImageDraw.Draw(img)
        
        # Get text bounding box
        bbox = draw.textbbox((0, 0), char, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        # Center the text
        x = (img_width - text_width) // 2
        y = (img_height - text_height) // 2
        
        # Draw black text on white background
        draw.text((x, y), char, fill='black', font=font)
        
        # Crop to target size (center crop)
        left = (img_width - target_width) // 2
        top = (img_height - target_height) // 2
        img = img.crop((left, top, left + target_width, top + target_height))
        
        # Convert to grayscale then to monochrome
        img = img.convert('L')
        
        # Convert to 1-bit monochrome (threshold at 128)
        img = img.point(lambda x: 0 if x < 128 else 255, mode='1')
        
        # Convert to packed bitmap data
        bitmap_data = []
        for y in range(target_height):
            for x in range(0, target_width, 8):
                byte_val = 0
                for bit in range(8):
                    if x + bit < target_width:
                        pixel = img.getpixel((x + bit, y))
                        if pixel == 0:  # Black pixel
                            byte_val |= (1 << (7 - bit))
                bitmap_data.append(byte_val)
        
        return bytes(bitmap_data)
    
    def generate_digit_font(self, output_path: str) -> bool:
        """
        Generate 24x48 digit font
        Characters: 0-9, -, .
        """
        print("🔢 Generating 24×48 digit font...")
        
        # Character set for digits
        digit_chars = "0123456789-."
        char_width = 24
        char_height = 48
        
        # Load font (larger size for better quality at 24x48)
        font = self._find_system_font(36)
        if not font:
            return False
        
        characters = []
        bitmap_data = b''
        current_offset = 4 + len(digit_chars) * 10  # Header + char info array
        
        for char in digit_chars:
            unicode_val = ord(char)
            
            # Render character bitmap
            char_bitmap = self._render_character(char, font, char_width, char_height)
            
            # Create character info
            char_info = CharacterInfo(unicode_val, char_width, char_height, current_offset)
            characters.append(char_info)
            
            # Append bitmap data
            bitmap_data += char_bitmap
            current_offset += len(char_bitmap)
            
            print(f"  ✓ Rendered '{char}' (U+{unicode_val:04X}): {len(char_bitmap)} bytes")
        
        # Write binary file
        try:
            with open(output_path, 'wb') as f:
                # Write header (character count)
                f.write(struct.pack('<I', len(characters)))
                
                # Write character info array
                for char_info in characters:
                    f.write(char_info.to_bytes())
                
                # Write bitmap data
                f.write(bitmap_data)
            
            print(f"✅ Digit font saved: {output_path} ({os.path.getsize(output_path)} bytes)")
            return True
            
        except Exception as e:
            print(f"❌ Failed to save digit font: {e}")
            return False
    
    def generate_ascii_font(self, output_path: str) -> bool:
        """
        Generate 16x24 ASCII font
        Characters: 32-126 (printable ASCII)
        """
        print("📝 Generating 16×24 ASCII font...")
        
        # Character set for ASCII (printable characters)
        ascii_chars = [chr(i) for i in range(32, 127)]  # 0x20 to 0x7E
        char_width = 16
        char_height = 24
        
        # Load font (appropriate size for 16x24)
        font = self._find_system_font(18)
        if not font:
            return False
        
        characters = []
        bitmap_data = b''
        current_offset = 4 + len(ascii_chars) * 10  # Header + char info array
        
        for char in ascii_chars:
            unicode_val = ord(char)
            
            # Render character bitmap
            char_bitmap = self._render_character(char, font, char_width, char_height)
            
            # Create character info
            char_info = CharacterInfo(unicode_val, char_width, char_height, current_offset)
            characters.append(char_info)
            
            # Append bitmap data
            bitmap_data += char_bitmap
            current_offset += len(char_bitmap)
            
            # Show progress for some characters
            if unicode_val % 10 == 0 or char in "AZaz09":
                print(f"  ✓ Rendered '{char}' (U+{unicode_val:04X}): {len(char_bitmap)} bytes")
        
        # Write binary file
        try:
            with open(output_path, 'wb') as f:
                # Write header (character count)
                f.write(struct.pack('<I', len(characters)))
                
                # Write character info array
                for char_info in characters:
                    f.write(char_info.to_bytes())
                
                # Write bitmap data
                f.write(bitmap_data)
            
            print(f"✅ ASCII font saved: {output_path} ({os.path.getsize(output_path)} bytes)")
            return True
            
        except Exception as e:
            print(f"❌ Failed to save ASCII font: {e}")
            return False


def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="Generate custom fonts for STM32G431CBU6")
    parser.add_argument("--output-dir", "-o", default=".", 
                       help="Output directory for font files")
    parser.add_argument("--font-name", "-f", default="Consolas",
                       help="Font name to use (default: Consolas)")
    parser.add_argument("--digit-only", action="store_true",
                       help="Generate only digit font")
    parser.add_argument("--ascii-only", action="store_true", 
                       help="Generate only ASCII font")
    
    args = parser.parse_args()
    
    # Create output directory if it doesn't exist
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Initialize generator
    generator = CustomFontGenerator(font_name=args.font_name)
    
    success = True
    
    # Generate digit font
    if not args.ascii_only:
        digit_output = os.path.join(args.output_dir, "digit_font_24x48.bin")
        if not generator.generate_digit_font(digit_output):
            success = False
    
    # Generate ASCII font
    if not args.digit_only:
        ascii_output = os.path.join(args.output_dir, "ascii_font_16x24.bin")
        if not generator.generate_ascii_font(ascii_output):
            success = False
    
    if success:
        print("\n🎉 Font generation completed successfully!")
    else:
        print("\n❌ Font generation failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
