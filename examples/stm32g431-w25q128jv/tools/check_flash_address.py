#!/usr/bin/env python3
"""
Check specific address in flash image to verify data
"""

import os
import struct

def check_flash_address():
    """Check the specific address where 'F' bitmap should be"""
    flash_image_path = '../w25q128jv_complete.bin'
    
    if not os.path.exists(flash_image_path):
        print(f"Flash image not found: {flash_image_path}")
        return
    
    # From STM32 log: Read font bitmap for 'F' (5x7, 7 bytes) from 0x00024279
    target_address = 0x00024279
    
    print(f"🔍 Checking Flash image at address 0x{target_address:08X}")
    
    with open(flash_image_path, 'rb') as f:
        f.seek(target_address)
        data = f.read(7)  # 'F' character should be 7 bytes
        
        print(f"Data at 0x{target_address:08X}: {list(data)}")
        print(f"Hex: {' '.join(f'{b:02X}' for b in data)}")
        
        # Expected data for 'F' character
        expected = [240, 128, 128, 224, 128, 128, 128]  # [0xF0, 0x80, 0x80, 0xE0, 0x80, 0x80, 0x80]
        
        print(f"Expected:     {expected}")
        print(f"Expected hex: {' '.join(f'{b:02X}' for b in expected)}")
        
        if list(data) == expected:
            print("✅ Data matches expected 'F' character!")
        else:
            print("❌ Data does NOT match!")
            
        # Also check what STM32 reported reading
        stm32_reported = [112, 0, 0, 96, 0, 0, 0]  # [0x70, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00]
        print(f"STM32 read:   {stm32_reported}")
        print(f"STM32 hex:    {' '.join(f'{b:02X}' for b in stm32_reported)}")
        
        # Let's also check a few bytes before and after to see if there's an offset issue
        print(f"\n🔍 Context around address 0x{target_address:08X}:")
        f.seek(target_address - 8)
        context = f.read(24)  # 8 before + 7 target + 8 after
        
        for i, byte in enumerate(context):
            addr = target_address - 8 + i
            marker = " <-- TARGET" if 8 <= i < 15 else ""
            print(f"  0x{addr:08X}: 0x{byte:02X} ({byte:3d}){marker}")

if __name__ == "__main__":
    check_flash_address()
