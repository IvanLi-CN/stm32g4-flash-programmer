#!/usr/bin/env python3
"""
Generate RGB565 bitmap for STM32G4 Flash Programmer boot screen
Creates a 320x172 boot screen bitmap in RGB565 format for W25Q128JV flash memory
"""

import os
import sys
import struct
from PIL import Image, ImageDraw, ImageFont

def rgb888_to_rgb565(r, g, b):
    """Convert RGB888 to RGB565 format (little-endian)"""
    r5 = (r >> 3) & 0x1F
    g6 = (g >> 2) & 0x3F  
    b5 = (b >> 3) & 0x1F
    rgb565 = (r5 << 11) | (g6 << 5) | b5
    # Return as little-endian bytes
    return struct.pack('<H', rgb565)

def create_boot_screen(output_path, width=320, height=172):
    """Create boot screen bitmap using PIL"""
    print(f"Creating boot screen bitmap {width}x{height}...")

    try:
        # Create image with blue gradient background
        image = Image.new('RGB', (width, height), (30, 58, 138))  # Dark blue
        draw = ImageDraw.Draw(image)

        # Create gradient background
        for y in range(height):
            # Gradient from dark blue to lighter blue
            ratio = y / height
            r = int(30 + (59 - 30) * ratio)    # 30 -> 59
            g = int(58 + (130 - 58) * ratio)   # 58 -> 130
            b = int(138 + (246 - 138) * ratio) # 138 -> 246
            draw.line([(0, y), (width, y)], fill=(r, g, b))

        # Draw circuit pattern overlay
        for x in range(0, width, 40):
            for y in range(0, height, 40):
                # Small circles
                draw.ellipse([x+18, y+18, x+22, y+22], fill=(96, 165, 250, 80))
                # Grid lines (faint)
                if x < width - 40:
                    draw.line([x+20, y+20, x+60, y+20], fill=(96, 165, 250, 50))
                if y < height - 40:
                    draw.line([x+20, y+20, x+20, y+60], fill=(96, 165, 250, 50))

        # Draw main microcontroller chip
        chip_x, chip_y = width//2, 50
        chip_w, chip_h = 50, 40

        # Chip body
        draw.rectangle([chip_x-chip_w//2, chip_y-chip_h//2,
                       chip_x+chip_w//2, chip_y+chip_h//2],
                      fill=(248, 250, 252), outline=(226, 232, 240), width=2)

        # Chip pins
        pin_color = (148, 163, 184)
        # Left pins
        for i, pin_y in enumerate([-15, -5, 5, 15]):
            draw.line([chip_x-chip_w//2, chip_y+pin_y, chip_x-chip_w//2-5, chip_y+pin_y],
                     fill=pin_color, width=2)
        # Right pins
        for i, pin_y in enumerate([-15, -5, 5, 15]):
            draw.line([chip_x+chip_w//2, chip_y+pin_y, chip_x+chip_w//2+5, chip_y+pin_y],
                     fill=pin_color, width=2)
        # Top pins
        for i, pin_x in enumerate([-15, -5, 5, 15]):
            draw.line([chip_x+pin_x, chip_y-chip_h//2, chip_x+pin_x, chip_y-chip_h//2-5],
                     fill=pin_color, width=2)
        # Bottom pins
        for i, pin_x in enumerate([-15, -5, 5, 15]):
            draw.line([chip_x+pin_x, chip_y+chip_h//2, chip_x+pin_x, chip_y+chip_h//2+5],
                     fill=pin_color, width=2)

        # Flash memory chip
        flash_x, flash_y = 80, 120
        flash_w, flash_h = 30, 16
        draw.rectangle([flash_x-flash_w//2, flash_y-flash_h//2,
                       flash_x+flash_w//2, flash_y+flash_h//2],
                      fill=(251, 191, 36), outline=(245, 158, 11), width=1)

        # Connection line (simple line instead of arc to avoid coordinate issues)
        draw.line([chip_x, chip_y+chip_h//2, flash_x, flash_y-flash_h//2],
                 fill=(96, 165, 250), width=2)

        # Status indicator (green circle)
        status_x, status_y = 280, 30
        draw.ellipse([status_x-6, status_y-6, status_x+6, status_y+6], fill=(16, 185, 129))
        draw.ellipse([status_x-3, status_y-3, status_x+3, status_y+3], fill=(52, 211, 153))

        # Try to load a font, fall back to default if not available
        try:
            # Try to use a system font
            title_font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", 16)
            subtitle_font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", 10)
            small_font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", 8)
        except:
            # Fall back to default font
            title_font = ImageFont.load_default()
            subtitle_font = ImageFont.load_default()
            small_font = ImageFont.load_default()

        # Title text
        title = "STM32G4 Flash Programmer"
        title_bbox = draw.textbbox((0, 0), title, font=title_font)
        title_w = title_bbox[2] - title_bbox[0]
        draw.text((width//2 - title_w//2, 140), title, fill=(248, 250, 252), font=title_font)

        # Subtitle
        subtitle = "W25Q128JV Resource Manager"
        subtitle_bbox = draw.textbbox((0, 0), subtitle, font=subtitle_font)
        subtitle_w = subtitle_bbox[2] - subtitle_bbox[0]
        draw.text((width//2 - subtitle_w//2, 158), subtitle, fill=(203, 213, 225), font=subtitle_font)

        # Version info
        draw.text((10, 165), "v1.0", fill=(148, 163, 184), font=small_font)

        # Labels for chips
        draw.text((chip_x-12, chip_y-3), "STM32", fill=(30, 64, 175), font=small_font)
        draw.text((flash_x-12, flash_y-3), "FLASH", fill=(146, 64, 14), font=small_font)

        print(f"Image created: {image.size}")

        # Convert to RGB565 format
        rgb565_data = bytearray()
        pixels = image.load()

        for y in range(height):
            for x in range(width):
                r, g, b = pixels[x, y]
                rgb565_bytes = rgb888_to_rgb565(r, g, b)
                rgb565_data.extend(rgb565_bytes)

        # Write to output file
        with open(output_path, 'wb') as f:
            f.write(rgb565_data)

        file_size = len(rgb565_data)
        expected_size = width * height * 2

        print(f"✓ Boot screen created successfully!")
        print(f"Output file: {output_path}")
        print(f"File size: {file_size} bytes (expected: {expected_size} bytes)")
        print(f"Format: RGB565 little-endian")

        return True

    except Exception as e:
        print(f"Error creating boot screen: {e}")
        return False

def create_test_pattern(output_path, width=320, height=172):
    """Create a simple test pattern if SVG conversion fails"""
    print(f"Creating test pattern {width}x{height}...")
    
    rgb565_data = bytearray()
    
    for y in range(height):
        for x in range(width):
            # Create a simple gradient pattern
            r = (x * 255) // width
            g = (y * 255) // height
            b = 128
            
            rgb565_bytes = rgb888_to_rgb565(r, g, b)
            rgb565_data.extend(rgb565_bytes)
    
    with open(output_path, 'wb') as f:
        f.write(rgb565_data)
    
    print(f"✓ Test pattern created: {output_path}")
    print(f"File size: {len(rgb565_data)} bytes")

def main():
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(os.path.dirname(script_dir), 'assets')

    # Output path
    output_path = os.path.join(assets_dir, 'boot_screen_320x172.bin')

    print("=== Boot Screen Generator ===")
    print(f"Output bitmap: {output_path}")

    # Create assets directory if it doesn't exist
    os.makedirs(assets_dir, exist_ok=True)

    # Create boot screen
    if create_boot_screen(output_path):
        print("\n=== Boot Screen Generation Complete ===")
        print("The RGB565 bitmap is ready for programming to W25Q128JV flash memory.")
        print("Use this bitmap as the boot screen for your STM32G4 project.")
        return 0
    else:
        print("\n=== Boot Screen Generation Failed ===")
        print("Creating test pattern instead...")
        create_test_pattern(output_path)
        return 1

if __name__ == "__main__":
    exit(main())
