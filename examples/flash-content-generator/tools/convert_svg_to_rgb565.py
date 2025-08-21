#!/usr/bin/env python3
"""
Convert SVG boot screens to RGB565 format for STM32G4 Flash Programmer
Converts multiple SVG designs to 320x172 RGB565 bitmaps for W25Q128JV flash memory
"""

import os
import sys
import struct
from PIL import Image, ImageDraw
import subprocess

def rgb888_to_rgb565(r, g, b):
    """Convert RGB888 to RGB565 format (little-endian)"""
    r5 = (r >> 3) & 0x1F
    g6 = (g >> 2) & 0x3F  
    b5 = (b >> 3) & 0x1F
    rgb565 = (r5 << 11) | (g6 << 5) | b5
    # Return as little-endian bytes
    return struct.pack('<H', rgb565)

def svg_to_png(svg_path, png_path, width=320, height=172):
    """Convert SVG to PNG using system tools"""
    try:
        # Try using cairosvg first (if available)
        try:
            import cairosvg
            cairosvg.svg2png(url=svg_path, write_to=png_path, 
                           output_width=width, output_height=height)
            return True
        except ImportError:
            pass
        
        # Try using Inkscape
        try:
            cmd = [
                'inkscape',
                '--export-type=png',
                f'--export-filename={png_path}',
                f'--export-width={width}',
                f'--export-height={height}',
                svg_path
            ]
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                return True
        except FileNotFoundError:
            pass
        
        # Try using rsvg-convert
        try:
            cmd = [
                'rsvg-convert',
                '-w', str(width),
                '-h', str(height),
                '-f', 'png',
                '-o', png_path,
                svg_path
            ]
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                return True
        except FileNotFoundError:
            pass
        
        print(f"Warning: No SVG converter found. Please install cairosvg, inkscape, or librsvg")
        return False
        
    except Exception as e:
        print(f"Error converting SVG to PNG: {e}")
        return False

def convert_svg_to_rgb565(svg_path, output_path, width=320, height=172):
    """Convert SVG file to RGB565 bitmap"""
    try:
        print(f"Converting {os.path.basename(svg_path)}...")
        
        # Create temporary PNG file
        temp_png = svg_path.replace('.svg', '_temp.png')
        
        # Convert SVG to PNG
        if not svg_to_png(svg_path, temp_png, width, height):
            print(f"Failed to convert SVG to PNG: {svg_path}")
            return False
        
        # Load PNG and convert to RGB565
        try:
            image = Image.open(temp_png)
            
            # Ensure image is RGB mode
            if image.mode != 'RGB':
                image = image.convert('RGB')
            
            # Resize if necessary
            if image.size != (width, height):
                image = image.resize((width, height), Image.Resampling.LANCZOS)
            
            print(f"Image loaded: {image.size}, mode: {image.mode}")
            
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
            
            print(f"✓ Converted successfully!")
            print(f"  Output: {output_path}")
            print(f"  Size: {file_size} bytes (expected: {expected_size} bytes)")
            
            # Clean up temporary file
            if os.path.exists(temp_png):
                os.remove(temp_png)
            
            return True
            
        except Exception as e:
            print(f"Error processing image: {e}")
            # Clean up temporary file
            if os.path.exists(temp_png):
                os.remove(temp_png)
            return False
            
    except Exception as e:
        print(f"Error converting SVG: {e}")
        return False

def convert_all_boot_screens():
    """Convert all boot screen SVG files to RGB565 format"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(os.path.dirname(script_dir), 'assets')
    
    # List of boot screen designs
    designs = [
        'boot_screen_professional',
        'boot_screen_minimal', 
        'boot_screen_pcb',
        'boot_screen_energy',
        'boot_screen_dashboard'
    ]
    
    print("=== USB PD Sink Boot Screen Converter ===")
    print(f"Assets directory: {assets_dir}")
    print(f"Converting {len(designs)} boot screen designs...\n")
    
    success_count = 0
    
    for design in designs:
        svg_path = os.path.join(assets_dir, f'{design}.svg')
        bin_path = os.path.join(assets_dir, f'{design}.bin')
        
        if not os.path.exists(svg_path):
            print(f"⚠️  SVG file not found: {svg_path}")
            continue
        
        if convert_svg_to_rgb565(svg_path, bin_path):
            success_count += 1
        else:
            print(f"❌ Failed to convert: {design}")
        
        print()  # Empty line for readability
    
    print(f"=== Conversion Complete ===")
    print(f"Successfully converted: {success_count}/{len(designs)} designs")
    
    if success_count > 0:
        print("\n📁 Generated files:")
        for design in designs:
            bin_path = os.path.join(assets_dir, f'{design}.bin')
            if os.path.exists(bin_path):
                size = os.path.getsize(bin_path)
                print(f"  {design}.bin ({size:,} bytes)")
        
        print("\n🚀 Usage in STM32 project:")
        print("1. Copy the .bin files to your flash memory")
        print("2. Use the display manager to load and show the boot screen")
        print("3. Example: display_manager.show_boot_screen(&mut flash_manager)")
    
    return success_count

def main():
    """Main function"""
    if len(sys.argv) > 1:
        # Convert specific file
        svg_path = sys.argv[1]
        if not svg_path.endswith('.svg'):
            print("Error: Input file must be an SVG file")
            return 1
        
        output_path = svg_path.replace('.svg', '.bin')
        if len(sys.argv) > 2:
            output_path = sys.argv[2]
        
        if convert_svg_to_rgb565(svg_path, output_path):
            return 0
        else:
            return 1
    else:
        # Convert all boot screens
        success_count = convert_all_boot_screens()
        return 0 if success_count > 0 else 1

if __name__ == "__main__":
    exit(main())
