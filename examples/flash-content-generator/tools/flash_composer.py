#!/usr/bin/env python3
"""
Flash Composer for W25Q128JV
Combines all resources into a single 16MB flash image
"""

import os
import sys
import json
import struct
import shutil

class FlashComposer:
    """Compose complete flash image for W25Q128JV"""
    
    FLASH_SIZE = 16 * 1024 * 1024  # 16MB
    
    def __init__(self, assets_dir):
        self.assets_dir = assets_dir
        self.flash_image = bytearray(self.FLASH_SIZE)
        
        # Initialize with 0xFF (erased flash state)
        for i in range(self.FLASH_SIZE):
            self.flash_image[i] = 0xFF
    
    def load_resource_layout(self):
        """Load resource layout from JSON"""
        layout_path = os.path.join(self.assets_dir, 'resource_layout.json')
        
        with open(layout_path, 'r') as f:
            layout = json.load(f)
        
        return layout['resources']
    
    def add_resource(self, name, address, file_path):
        """Add a resource file to the flash image"""
        if not os.path.exists(file_path):
            print(f"⚠️  Resource file not found: {file_path}")
            return False
        
        file_size = os.path.getsize(file_path)
        
        if address + file_size > self.FLASH_SIZE:
            print(f"❌ Resource {name} exceeds flash size!")
            return False
        
        print(f"📄 Adding {name}:")
        print(f"   File: {os.path.basename(file_path)}")
        print(f"   Address: 0x{address:08X}")
        print(f"   Size: {file_size:,} bytes")
        
        with open(file_path, 'rb') as f:
            data = f.read()
            
        # Copy data to flash image
        for i, byte in enumerate(data):
            self.flash_image[address + i] = byte
        
        print(f"   ✅ Added successfully")
        return True
    
    def compose_flash_image(self):
        """Compose the complete flash image"""
        print("🔧 W25Q128JV Flash Composer")
        print("=" * 50)
        
        # Load resource layout
        resources = self.load_resource_layout()
        
        # Add each resource
        success_count = 0
        total_used = 0
        
        for resource in resources:
            name = resource['name']
            address = resource['address']
            
            # Skip empty/reserved areas
            if name in ['ui_graphics', 'app_data', 'user_config', 'log_storage',
                       'firmware_update', 'reserved']:
                print(f"⏭️  Skipping {name} (empty area)")
                continue

            # Map resource names to files
            file_mapping = {
                'boot_screen': 'boot_screen_320x172.bin',
                'font_bitmap_12px': 'font_output/font_bitmap_12px.bin',
                'font_bitmap_16px': 'font_output/font_bitmap_16px.bin',
                'arial_font_16x24': 'font_output/arial_font_16x24.bin',
                'grotesk_font_24x48': 'font_output/grotesk_font_24x48.bin'
            }
            
            if name in file_mapping:
                file_path = os.path.join(self.assets_dir, file_mapping[name])
                
                if self.add_resource(name, address, file_path):
                    success_count += 1
                    if os.path.exists(file_path):
                        total_used += os.path.getsize(file_path)
                
                print()  # Empty line for readability
        
        print(f"📊 Composition Summary:")
        print(f"   Resources added: {success_count}")
        print(f"   Total data: {total_used:,} bytes")
        print(f"   Flash utilization: {(total_used/self.FLASH_SIZE)*100:.2f}%")
        
        return success_count > 0
    
    def save_flash_image(self, output_path):
        """Save the complete flash image"""
        print(f"💾 Saving flash image to: {output_path}")
        
        with open(output_path, 'wb') as f:
            f.write(self.flash_image)
        
        file_size = os.path.getsize(output_path)
        print(f"   ✅ Saved {file_size:,} bytes ({file_size // (1024*1024)} MB)")
        
        return True
    
    def verify_flash_image(self, output_path):
        """Verify the flash image"""
        print(f"🔍 Verifying flash image...")
        
        # Check file size
        file_size = os.path.getsize(output_path)
        if file_size != self.FLASH_SIZE:
            print(f"❌ Wrong file size: {file_size} (expected {self.FLASH_SIZE})")
            return False
        
        # Check boot screen
        with open(output_path, 'rb') as f:
            # Check boot screen signature (first few bytes)
            boot_data = f.read(16)
            if all(b == 0xFF for b in boot_data):
                print("⚠️  Boot screen area appears empty")
            else:
                print("✅ Boot screen data present")
            
            # Check 12px font data
            f.seek(0x00020000)  # 12px Font address
            font_header_12px = f.read(4)
            if len(font_header_12px) == 4:
                char_count_12px = struct.unpack('<I', font_header_12px)[0]
                if char_count_12px == 27678:
                    print("✅ 12px Font data verified (27678 characters)")
                else:
                    print(f"⚠️  12px Font character count: {char_count_12px} (expected 27678)")

            # Check 16px font data
            f.seek(0x00120000)  # 16px Font address
            font_header_16px = f.read(4)
            if len(font_header_16px) == 4:
                char_count_16px = struct.unpack('<I', font_header_16px)[0]
                if char_count_16px == 27678:
                    print("✅ 16px Font data verified (27678 characters)")
                else:
                    print(f"⚠️  16px Font character count: {char_count_16px} (expected 27678)")
            
            # Check empty areas are 0xFF
            f.seek(0x00220000)  # UI graphics area
            empty_sample = f.read(1024)
            if all(b == 0xFF for b in empty_sample):
                print("✅ Empty areas properly initialized (0xFF)")
            else:
                print("⚠️  Empty areas contain data")
        
        print("✅ Flash image verification complete")
        return True

    def copy_to_webapp(self, source_path):
        """Copy firmware to web-app directory for preview"""
        # Get the web-app directory path
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_dir = os.path.dirname(script_dir)
        webapp_dir = os.path.join(project_dir, 'web-app')

        if not os.path.exists(webapp_dir):
            print(f"⚠️  Web-app directory not found: {webapp_dir}")
            return False

        destination_path = os.path.join(webapp_dir, os.path.basename(source_path))

        try:
            print(f"📋 Copying firmware to web-app directory...")
            shutil.copy2(source_path, destination_path)

            # Verify copy
            if os.path.exists(destination_path):
                dest_size = os.path.getsize(destination_path)
                src_size = os.path.getsize(source_path)

                if dest_size == src_size:
                    print(f"   ✅ Successfully copied to: {destination_path}")
                    print(f"   📏 Size: {dest_size:,} bytes")
                    return True
                else:
                    print(f"   ❌ Size mismatch: {dest_size} != {src_size}")
                    return False
            else:
                print(f"   ❌ Copy failed: destination file not found")
                return False

        except Exception as e:
            print(f"   ❌ Copy failed: {e}")
            return False


def main():
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(os.path.dirname(script_dir), 'assets')
    output_dir = assets_dir  # 修改：将固件生成到assets目录
    
    print("🔧 W25Q128JV Flash Composer")
    print("=" * 50)
    print(f"Assets directory: {assets_dir}")
    print(f"Output directory: {output_dir}")
    print()
    
    # Create composer
    composer = FlashComposer(assets_dir)
    
    # Compose flash image
    if not composer.compose_flash_image():
        print("❌ Failed to compose flash image")
        return 1
    
    # Save flash image
    output_path = os.path.join(output_dir, 'w25q128jv_complete.bin')
    if not composer.save_flash_image(output_path):
        print("❌ Failed to save flash image")
        return 1
    
    # Verify flash image
    if not composer.verify_flash_image(output_path):
        print("❌ Flash image verification failed")
        return 1

    # Copy to web-app directory for preview
    print()
    if composer.copy_to_webapp(output_path):
        print("✅ Firmware copied to web-app directory")
    else:
        print("⚠️  Failed to copy firmware to web-app directory")

    print()
    print("🎉 Flash composition complete!")
    print(f"📁 Complete flash image: {output_path}")
    print(f"📏 Size: 16 MB (16,777,216 bytes)")
    print()
    print("💡 Programming commands:")
    print("   # Program complete image (recommended)")
    print("   flash-programmer-tool --port /dev/ttyACM0 write \\")
    print(f"     --file {os.path.basename(output_path)} --address 0x000000 --erase --verify")
    print()
    print("   # Verify flash information")
    print("   flash-programmer-tool --port /dev/ttyACM0 info")
    print()
    print("🌐 Web preview available at: web-app/index.html")

    return 0

if __name__ == "__main__":
    exit(main())
