#!/usr/bin/env python3
"""
Complete Flash Firmware Generation Script
Generates a complete 16MB Flash image with all resources and deploys to web-app
"""

import os
import sys
import subprocess
import time

def run_command(cmd, description):
    """Run a command and handle errors"""
    print(f"🔧 {description}...")
    try:
        result = subprocess.run(cmd, shell=True, check=True, capture_output=True, text=True)
        print(f"   ✅ {description} completed")
        if result.stdout.strip():
            print(f"   📄 Output: {result.stdout.strip()}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"   ❌ {description} failed: {e}")
        if e.stderr:
            print(f"   📄 Error: {e.stderr.strip()}")
        return False

def main():
    print("🚀 STM32G4 Flash Firmware Generator")
    print("=" * 50)
    
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    tools_dir = os.path.join(script_dir, 'tools')
    assets_dir = os.path.join(script_dir, 'assets')
    
    print(f"📁 Working directory: {script_dir}")
    print(f"🔧 Tools directory: {tools_dir}")
    print(f"📦 Assets directory: {assets_dir}")
    print()
    
    # Change to tools directory
    os.chdir(tools_dir)
    
    # Step 1: Generate 12px font
    print("📝 Step 1: Generate 12px Font Bitmap")
    if not run_command(
        f"python3 font_converter.py ../assets/VonwaonBitmap-12px.ttf --output ../assets/font_output/",
        "Converting 12px font to bitmap"
    ):
        return 1
    print()
    
    # Step 2: Generate 16px font
    print("📝 Step 2: Generate 16px Font Bitmap")
    if not run_command(
        f"python3 font_converter.py ../assets/VonwaonBitmap-16px.ttf --output ../assets/font_output/",
        "Converting 16px font to bitmap"
    ):
        return 1
    print()
    
    # Step 3: Generate boot screen (if SVG exists)
    boot_screen_svg = os.path.join(assets_dir, 'boot_screen.svg')
    if os.path.exists(boot_screen_svg):
        print("📝 Step 3: Generate Boot Screen")
        if not run_command(
            f"python3 svg_to_rgb565.py ../assets/boot_screen.svg --output ../assets/boot_screen_320x172.bin --width 320 --height 172",
            "Converting SVG to RGB565 boot screen"
        ):
            return 1
        print()
    else:
        print("📝 Step 3: Boot Screen (using existing binary)")
        print("   ⏭️  SVG not found, using existing boot_screen_320x172.bin")
        print()
    
    # Step 4: Compose complete Flash image
    print("📝 Step 4: Compose Complete Flash Image")
    if not run_command(
        "python3 flash_composer.py",
        "Composing complete Flash image with auto-deployment"
    ):
        return 1
    print()
    
    # Final summary
    output_file = os.path.join(script_dir, 'w25q128jv_complete.bin')
    webapp_file = os.path.join(script_dir, 'web-app', 'w25q128jv_complete.bin')
    
    print("🎉 Flash Firmware Generation Complete!")
    print("=" * 50)
    
    if os.path.exists(output_file):
        file_size = os.path.getsize(output_file)
        print(f"📁 Main firmware: {output_file}")
        print(f"📏 Size: {file_size:,} bytes ({file_size // (1024*1024)} MB)")
    
    if os.path.exists(webapp_file):
        print(f"🌐 Web preview: {webapp_file}")
        print(f"🔗 Open: web-app/index.html")
    
    print()
    print("💡 Next steps:")
    print("   1. Program the firmware to your W25Q128JV Flash")
    print("   2. Open web-app/index.html to preview the content")
    print("   3. Use the firmware in your STM32G4 project")
    
    return 0

if __name__ == "__main__":
    exit(main())
