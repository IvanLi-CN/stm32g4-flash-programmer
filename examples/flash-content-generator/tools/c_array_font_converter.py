#!/usr/bin/env python3
"""
C Array Font Converter
Converts C array format fonts to standard binary format for the PD-Sink project.

Author: AI Assistant (白羽)
Date: 2025-01-20
"""

import os
import sys
import re
import struct
import argparse
from typing import List, Tuple, Dict, Optional

class CArrayFontConverter:
    """Convert C array format fonts to standard binary format"""
    
    def __init__(self):
        self.verbose = False
    
    def log(self, message: str, level: str = "INFO"):
        """Log message with level"""
        if self.verbose or level in ["ERROR", "WARNING"]:
            print(f"[{level}] {message}")
    
    def parse_c_array_file(self, file_path: str) -> Dict:
        """Parse C array font file and extract font data"""
        self.log(f"Parsing C array file: {file_path}")
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            raise Exception(f"Failed to read file {file_path}: {e}")
        
        # Extract array name and data using regex
        array_pattern = r'fontdatatype\s+(\w+)\[(\d+)\]\s*PROGMEM\s*=\s*\{([^}]+)\}'
        match = re.search(array_pattern, content, re.DOTALL)
        
        if not match:
            raise Exception("Could not find font array in C file")
        
        array_name = match.group(1)
        array_size = int(match.group(2))
        array_data_str = match.group(3)
        
        self.log(f"Found array: {array_name}, size: {array_size}")
        
        # Parse array data
        hex_values = re.findall(r'0x([0-9A-Fa-f]{2})', array_data_str)
        if not hex_values:
            raise Exception("Could not extract hex values from array")
        
        array_data = [int(val, 16) for val in hex_values]
        
        if len(array_data) != array_size:
            self.log(f"Warning: Expected {array_size} bytes, got {len(array_data)}", "WARNING")
        
        # Extract font parameters from first 4 bytes
        if len(array_data) < 4:
            raise Exception("Array too short, missing font parameters")
        
        font_width = array_data[0]
        font_height = array_data[1]
        start_char = array_data[2]
        end_char = array_data[3]
        
        char_count = end_char - start_char + 1
        char_bitmap_size = (font_width * font_height + 7) // 8  # Round up to bytes
        
        self.log(f"Font parameters: {font_width}x{font_height}, chars {start_char}-{end_char} ({char_count} chars)")
        self.log(f"Character bitmap size: {char_bitmap_size} bytes")
        
        # Extract bitmap data (skip first 4 parameter bytes)
        bitmap_data = array_data[4:]
        expected_bitmap_size = char_count * char_bitmap_size
        
        if len(bitmap_data) < expected_bitmap_size:
            self.log(f"Warning: Expected {expected_bitmap_size} bitmap bytes, got {len(bitmap_data)}", "WARNING")
        
        return {
            'array_name': array_name,
            'font_width': font_width,
            'font_height': font_height,
            'start_char': start_char,
            'end_char': end_char,
            'char_count': char_count,
            'char_bitmap_size': char_bitmap_size,
            'bitmap_data': bitmap_data[:expected_bitmap_size]  # Trim to expected size
        }
    
    def convert_to_standard_format(self, font_data: Dict) -> bytes:
        """Convert C array format to standard binary format"""
        self.log("Converting to standard binary format")
        
        char_count = font_data['char_count']
        font_width = font_data['font_width']
        font_height = font_data['font_height']
        start_char = font_data['start_char']
        char_bitmap_size = font_data['char_bitmap_size']
        bitmap_data = font_data['bitmap_data']
        
        # Build binary data
        binary_data = bytearray()
        
        # 1. Character count (4 bytes, little-endian)
        binary_data.extend(struct.pack('<I', char_count))
        
        # 2. Character information array (char_count * 10 bytes)
        char_info_size = char_count * 10
        bitmap_offset = 4 + char_info_size  # After header and char info
        
        for i in range(char_count):
            char_code = start_char + i
            
            # Character info: 4 bytes code + 1 byte width + 1 byte height + 4 bytes offset
            binary_data.extend(struct.pack('<I', char_code))  # Unicode code point
            binary_data.extend(struct.pack('B', font_width))   # Character width
            binary_data.extend(struct.pack('B', font_height))  # Character height
            binary_data.extend(struct.pack('<I', bitmap_offset + i * char_bitmap_size))  # Bitmap offset
        
        # 3. Bitmap data
        binary_data.extend(bitmap_data)
        
        self.log(f"Generated binary data: {len(binary_data)} bytes")
        self.log(f"  - Header: 4 bytes")
        self.log(f"  - Character info: {char_info_size} bytes")
        self.log(f"  - Bitmap data: {len(bitmap_data)} bytes")
        
        return bytes(binary_data)
    
    def convert_font(self, input_path: str, output_path: str) -> bool:
        """Convert a single font file"""
        try:
            # Parse C array file
            font_data = self.parse_c_array_file(input_path)
            
            # Convert to standard format
            binary_data = self.convert_to_standard_format(font_data)
            
            # Create output directory if needed
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            
            # Write binary file
            with open(output_path, 'wb') as f:
                f.write(binary_data)
            
            self.log(f"Successfully converted {input_path} -> {output_path}")
            self.log(f"Output file size: {len(binary_data)} bytes")
            
            return True
            
        except Exception as e:
            self.log(f"Error converting {input_path}: {e}", "ERROR")
            return False

def main():
    parser = argparse.ArgumentParser(description='Convert C array fonts to standard binary format')
    parser.add_argument('input', help='Input C array font file')
    parser.add_argument('output', help='Output binary font file')
    parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    converter = CArrayFontConverter()
    converter.verbose = args.verbose
    
    if not os.path.exists(args.input):
        print(f"Error: Input file {args.input} does not exist")
        return 1
    
    success = converter.convert_font(args.input, args.output)
    return 0 if success else 1

if __name__ == '__main__':
    sys.exit(main())
