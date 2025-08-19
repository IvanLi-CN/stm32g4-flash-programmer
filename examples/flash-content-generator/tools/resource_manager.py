#!/usr/bin/env python3
"""
Resource Manager for W25Q128JV Flash Memory
Plans and manages resource layout for STM32G4 Flash Programmer
"""

import os
import sys
import struct
import json

class W25Q128JVResourceManager:
    """Resource manager for W25Q128JV 16MB flash memory"""
    
    # W25Q128JV specifications
    FLASH_SIZE = 16 * 1024 * 1024  # 16MB
    SECTOR_SIZE = 4096             # 4KB sectors
    PAGE_SIZE = 256                # 256 byte pages
    BLOCK_SIZE = 64 * 1024         # 64KB blocks
    
    def __init__(self):
        self.memory_map = {}
        self.resources = []
        self.next_address = 0
        
    def add_resource(self, name, size, alignment=None, description=""):
        """Add a resource to the memory map"""
        if alignment:
            # Align to specified boundary
            self.next_address = (self.next_address + alignment - 1) // alignment * alignment
        
        resource = {
            'name': name,
            'address': self.next_address,
            'size': size,
            'end_address': self.next_address + size - 1,
            'description': description,
            'sectors': self._calculate_sectors(self.next_address, size)
        }
        
        self.resources.append(resource)
        self.memory_map[name] = resource
        self.next_address += size
        
        return resource
    
    def _calculate_sectors(self, address, size):
        """Calculate which sectors a resource spans"""
        start_sector = address // self.SECTOR_SIZE
        end_sector = (address + size - 1) // self.SECTOR_SIZE
        return list(range(start_sector, end_sector + 1))
    
    def get_memory_usage(self):
        """Get memory usage statistics"""
        used_bytes = sum(r['size'] for r in self.resources)
        free_bytes = self.FLASH_SIZE - used_bytes
        usage_percent = (used_bytes / self.FLASH_SIZE) * 100
        
        return {
            'total_bytes': self.FLASH_SIZE,
            'used_bytes': used_bytes,
            'free_bytes': free_bytes,
            'usage_percent': usage_percent,
            'total_sectors': self.FLASH_SIZE // self.SECTOR_SIZE,
            'used_sectors': len(set().union(*[r['sectors'] for r in self.resources]))
        }
    
    def generate_memory_map_text(self, output_path):
        """Generate text file with memory map information"""
        with open(output_path, 'w') as f:
            f.write("W25Q128JV Memory Map\n")
            f.write("===================\n\n")

            # Flash specifications
            f.write("Flash Specifications:\n")
            f.write(f"- Total Size: {self.FLASH_SIZE // (1024*1024)} MB (0x{self.FLASH_SIZE:08X} bytes)\n")
            f.write(f"- Sector Size: {self.SECTOR_SIZE} bytes (0x{self.SECTOR_SIZE:04X})\n")
            f.write(f"- Page Size: {self.PAGE_SIZE} bytes (0x{self.PAGE_SIZE:04X})\n")
            f.write(f"- Block Size: {self.BLOCK_SIZE // 1024} KB (0x{self.BLOCK_SIZE:08X})\n\n")

            # Resource addresses
            f.write("Resource Memory Map:\n")
            f.write("===================\n\n")
            for resource in self.resources:
                f.write(f"Resource: {resource['name']}\n")
                f.write(f"  Address: 0x{resource['address']:08X} - 0x{resource['end_address']:08X}\n")
                f.write(f"  Size: {resource['size']:,} bytes\n")
                f.write(f"  Sectors: {min(resource['sectors'])}-{max(resource['sectors'])}\n")
                if resource['description']:
                    f.write(f"  Description: {resource['description']}\n")
                f.write("\n")

            # Programming commands
            f.write("Flash Programming Commands:\n")
            f.write("===========================\n\n")
            f.write("# Program complete flash image (recommended)\n")
            f.write("flash-programmer-tool --port /dev/ttyACM0 write \\\n")
            f.write("  --file w25q128jv_complete.bin --address 0x000000 --erase --verify\n\n")

            for resource in self.resources:
                if resource['name'] in ['boot_screen', 'font_bitmap_12px', 'font_bitmap_16px']:
                    if resource['name'] == 'boot_screen':
                        filename = "boot_screen_320x172.bin"
                    elif resource['name'] == 'font_bitmap_12px':
                        filename = "font_output/font_bitmap_12px.bin"
                    elif resource['name'] == 'font_bitmap_16px':
                        filename = "font_output/font_bitmap_16px.bin"
                    else:
                        continue

                    f.write(f"# Program {resource['name']}\n")
                    f.write(f"flash-programmer-tool --port /dev/ttyACM0 write \\\n")
                    f.write(f"  --file {filename} --address 0x{resource['address']:08X} --erase --verify\n\n")

            f.write("# Verification commands\n")
            f.write("# Check flash information\n")
            f.write("flash-programmer-tool --port /dev/ttyACM0 info\n\n")
            f.write("# Read back and verify specific sections\n")
            f.write("flash-programmer-tool --port /dev/ttyACM0 read \\\n")
            f.write("  --file boot_screen_verify.bin --address 0x000000 --size 0x1AE00\n\n")
    
    def generate_resource_table(self, output_path):
        """Generate JSON resource table"""
        data = {
            'flash_info': {
                'model': 'W25Q128JV',
                'size_bytes': self.FLASH_SIZE,
                'sector_size': self.SECTOR_SIZE,
                'page_size': self.PAGE_SIZE,
                'block_size': self.BLOCK_SIZE
            },
            'memory_usage': self.get_memory_usage(),
            'resources': self.resources
        }
        
        with open(output_path, 'w') as f:
            json.dump(data, f, indent=2)
    
    def print_memory_map(self):
        """Print formatted memory map"""
        print("=" * 80)
        print("W25Q128JV Memory Map")
        print("=" * 80)
        
        usage = self.get_memory_usage()
        print(f"Flash Size: {usage['total_bytes']:,} bytes ({usage['total_bytes'] // (1024*1024)} MB)")
        print(f"Used: {usage['used_bytes']:,} bytes ({usage['usage_percent']:.1f}%)")
        print(f"Free: {usage['free_bytes']:,} bytes")
        print(f"Sectors: {usage['used_sectors']}/{usage['total_sectors']} used")
        print()
        
        print(f"{'Resource':<20} {'Address':<12} {'Size':<12} {'End':<12} {'Sectors':<15} {'Description'}")
        print("-" * 80)
        
        for resource in self.resources:
            sectors_str = f"{min(resource['sectors'])}-{max(resource['sectors'])}" if len(resource['sectors']) > 1 else str(resource['sectors'][0])
            print(f"{resource['name']:<20} "
                  f"0x{resource['address']:08X}  "
                  f"{resource['size']:8,}  "
                  f"0x{resource['end_address']:08X}  "
                  f"{sectors_str:<15} "
                  f"{resource['description']}")

def create_default_layout():
    """Create default resource layout for W25Q128JV"""
    rm = W25Q128JVResourceManager()
    
    # Boot screen (320x172 RGB565)
    boot_screen_size = 320 * 172 * 2  # 110,080 bytes
    rm.add_resource("boot_screen", boot_screen_size, 
                   alignment=rm.SECTOR_SIZE,
                   description="320x172 RGB565 boot screen bitmap")
    
    # Font bitmap data (WenQuanYi 12px converted to bitmap)
    font_size = 2 * 1024 * 1024  # 2MB for bitmap font data
    rm.add_resource("font_bitmap", font_size,
                   alignment=rm.BLOCK_SIZE,
                   description="WenQuanYi 12px bitmap font data (Chinese + ASCII)")
    
    # UI graphics area (icons, buttons, etc.)
    ui_graphics_size = 2 * 1024 * 1024  # 2MB
    rm.add_resource("ui_graphics", ui_graphics_size,
                   alignment=rm.BLOCK_SIZE,
                   description="UI graphics and icons")

    # Application data area
    app_data_size = 3 * 1024 * 1024  # 3MB
    rm.add_resource("app_data", app_data_size,
                   alignment=rm.BLOCK_SIZE,
                   description="Application data storage")
    
    # User configuration
    config_size = 64 * 1024  # 64KB
    rm.add_resource("user_config", config_size,
                   alignment=rm.BLOCK_SIZE,
                   description="User configuration and settings")

    # Log storage
    log_size = 128 * 1024  # 128KB
    rm.add_resource("log_storage", log_size,
                   alignment=rm.BLOCK_SIZE,
                   description="System and error logs")

    # Firmware update area
    firmware_size = 512 * 1024  # 512KB
    rm.add_resource("firmware_update", firmware_size,
                   alignment=rm.BLOCK_SIZE,
                   description="Firmware update storage")
    
    # Reserved area for future use
    remaining = rm.FLASH_SIZE - rm.next_address
    if remaining > 0:
        rm.add_resource("reserved", remaining,
                       alignment=rm.BLOCK_SIZE,
                       description="Reserved for future use")
    
    return rm

def main():
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    assets_dir = os.path.join(os.path.dirname(script_dir), 'assets')
    
    print("=== W25Q128JV Resource Manager ===")
    
    # Create default layout
    rm = create_default_layout()
    
    # Print memory map
    rm.print_memory_map()
    
    # Generate output files
    os.makedirs(assets_dir, exist_ok=True)

    memory_map_path = os.path.join(assets_dir, 'memory_map.txt')
    json_path = os.path.join(assets_dir, 'resource_layout.json')

    rm.generate_memory_map_text(memory_map_path)
    rm.generate_resource_table(json_path)

    print(f"\n✓ Generated memory map: {memory_map_path}")
    print(f"✓ Generated resource table: {json_path}")
    
    return 0

if __name__ == "__main__":
    exit(main())
