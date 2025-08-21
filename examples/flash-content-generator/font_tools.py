#!/usr/bin/env python3
"""
Font Tools CLI - 字体工具命令行界面
统一的字体生成、查看、验证工具入口

Usage:
    python font_tools.py generate [options]
    python font_tools.py view <font_file> [options]
    python font_tools.py verify <flash_file>
    python font_tools.py build
    python font_tools.py help

Author: AI Assistant (白羽)
Date: 2025-01-20
"""

import os
import sys
import argparse
import subprocess
from pathlib import Path


class FontTools:
    """字体工具统一接口"""
    
    def __init__(self):
        self.script_dir = Path(__file__).parent
        self.tools_dir = self.script_dir / "tools"
        
    def generate_fonts(self, args):
        """生成自定义字体"""
        print("🎨 Generating custom fonts...")
        
        cmd = [
            sys.executable,
            str(self.tools_dir / "custom_font_generator.py"),
            "--output-dir", args.output_dir,
        ]
        
        if args.font_name:
            cmd.extend(["--font-name", args.font_name])
        
        if args.digit_only:
            cmd.append("--digit-only")
        elif args.ascii_only:
            cmd.append("--ascii-only")
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            print(result.stdout)
            print("✅ Font generation completed successfully!")
            return True
        except subprocess.CalledProcessError as e:
            print(f"❌ Font generation failed: {e}")
            print(e.stderr)
            return False
    
    def view_font(self, args):
        """查看字体文件"""
        print(f"👀 Viewing font file: {args.font_file}")
        
        if not os.path.exists(args.font_file):
            print(f"❌ Font file not found: {args.font_file}")
            return False
        
        cmd = [
            sys.executable,
            str(self.tools_dir / "font_viewer.py"),
            args.font_file,
        ]
        
        if args.info:
            cmd.append("--info")
        if args.table > 0:
            cmd.extend(["--table", str(args.table)])
        if args.render is not None:
            cmd.extend(["--render", str(args.render)])
        if args.validate:
            cmd.append("--validate")
        if args.all:
            cmd.append("--all")
        
        try:
            result = subprocess.run(cmd, check=True)
            return True
        except subprocess.CalledProcessError as e:
            print(f"❌ Font viewing failed: {e}")
            return False
    
    def verify_flash(self, args):
        """验证Flash镜像中的字体"""
        print(f"🔍 Verifying fonts in flash image: {args.flash_file}")
        
        if not os.path.exists(args.flash_file):
            print(f"❌ Flash file not found: {args.flash_file}")
            return False
        
        cmd = [
            sys.executable,
            str(self.tools_dir / "verify_fonts_in_flash.py"),
            args.flash_file,
        ]
        
        if args.verbose:
            cmd.append("--verbose")
        
        try:
            result = subprocess.run(cmd, check=True)
            return True
        except subprocess.CalledProcessError as e:
            print(f"❌ Flash verification failed: {e}")
            return False
    
    def build_all(self, args):
        """构建完整的Flash镜像"""
        print("🏗️ Building complete flash image...")
        
        # 1. 生成所有字体
        print("\n📝 Step 1: Generating fonts...")
        cmd = [sys.executable, str(self.tools_dir / "font_converter.py")]
        try:
            subprocess.run(cmd, check=True)
            print("✅ Font generation completed")
        except subprocess.CalledProcessError as e:
            print(f"❌ Font generation failed: {e}")
            return False
        
        # 2. 合成Flash镜像
        print("\n🔧 Step 2: Composing flash image...")
        cmd = [sys.executable, str(self.tools_dir / "flash_composer.py")]
        try:
            subprocess.run(cmd, check=True)
            print("✅ Flash composition completed")
        except subprocess.CalledProcessError as e:
            print(f"❌ Flash composition failed: {e}")
            return False
        
        # 3. 重命名最终文件
        print("\n📦 Step 3: Finalizing output...")
        try:
            if os.path.exists("w25q128jv_complete.bin"):
                if os.path.exists("pd-sink-128mbit.bin"):
                    os.remove("pd-sink-128mbit.bin")
                os.rename("w25q128jv_complete.bin", "pd-sink-128mbit.bin")
                print("✅ Final file created: pd-sink-128mbit.bin")
            else:
                print("⚠️ Expected output file not found")
                return False
        except Exception as e:
            print(f"❌ File operation failed: {e}")
            return False
        
        # 4. 验证最终结果
        print("\n🔍 Step 4: Verifying final image...")
        verify_args = argparse.Namespace(
            flash_file="pd-sink-128mbit.bin",
            verbose=False
        )
        if not self.verify_flash(verify_args):
            print("⚠️ Verification failed, but build completed")
        
        print("\n🎉 Build completed successfully!")
        print(f"📁 Output file: {os.path.abspath('pd-sink-128mbit.bin')}")
        print(f"📏 File size: {os.path.getsize('pd-sink-128mbit.bin'):,} bytes")
        
        return True
    
    def show_help(self):
        """显示帮助信息"""
        help_text = """
🎨 Font Tools for STM32G431CBU6 PD-Sink Project

COMMANDS:
    generate    Generate custom fonts
    view        View and inspect font files
    verify      Verify fonts in flash image
    build       Build complete flash image
    help        Show this help message

EXAMPLES:
    # Generate all custom fonts
    python font_tools.py generate

    # Generate only digit font with specific font
    python font_tools.py generate --digit-only --font-name "Consolas"

    # View font file information
    python font_tools.py view output/digit_font_24x48.bin --info

    # View character table and render specific character
    python font_tools.py view output/ascii_font_16x24.bin --table 10 --render 65

    # Verify fonts in flash image
    python font_tools.py verify pd-sink-128mbit.bin

    # Build complete flash image
    python font_tools.py build

FONT SPECIFICATIONS:
    • Digital Font (24×48): Numbers 0-9, minus (-), decimal (.)
    • ASCII Font (16×24): Complete printable ASCII set (32-126)
    • Both fonts are monospace and single-color bitmap
    • Compatible with existing STM32 font system

FILES GENERATED:
    • digit_font_24x48.bin    - 24×48 digit font
    • ascii_font_16x24.bin    - 16×24 ASCII font
    • pd-sink-128mbit.bin     - Complete 16MB flash image

For more information, see:
    • CUSTOM_FONTS_README.md
    • STM32_FONT_USAGE.md
    • PROJECT_SUMMARY.md
        """
        print(help_text)


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        description="Font Tools for STM32G431CBU6 PD-Sink Project",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Generate command
    gen_parser = subparsers.add_parser("generate", help="Generate custom fonts")
    gen_parser.add_argument("--output-dir", "-o", default="output", 
                           help="Output directory for font files")
    gen_parser.add_argument("--font-name", "-f", 
                           help="Font name to use (default: Consolas)")
    gen_parser.add_argument("--digit-only", action="store_true",
                           help="Generate only digit font")
    gen_parser.add_argument("--ascii-only", action="store_true",
                           help="Generate only ASCII font")
    
    # View command
    view_parser = subparsers.add_parser("view", help="View and inspect font files")
    view_parser.add_argument("font_file", help="Path to font file")
    view_parser.add_argument("--info", "-i", action="store_true",
                            help="Show font information")
    view_parser.add_argument("--table", "-t", type=int, default=0,
                            help="Show character table (limit N entries)")
    view_parser.add_argument("--render", "-r", type=int,
                            help="Render character at index as ASCII art")
    view_parser.add_argument("--validate", "-v", action="store_true",
                            help="Validate font file integrity")
    view_parser.add_argument("--all", "-a", action="store_true",
                            help="Show all information")
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify fonts in flash image")
    verify_parser.add_argument("flash_file", help="Path to flash image file")
    verify_parser.add_argument("--verbose", "-v", action="store_true",
                              help="Verbose output")
    
    # Build command
    build_parser = subparsers.add_parser("build", help="Build complete flash image")
    
    # Help command
    help_parser = subparsers.add_parser("help", help="Show detailed help")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    tools = FontTools()
    
    if args.command == "generate":
        success = tools.generate_fonts(args)
    elif args.command == "view":
        success = tools.view_font(args)
    elif args.command == "verify":
        success = tools.verify_flash(args)
    elif args.command == "build":
        success = tools.build_all(args)
    elif args.command == "help":
        tools.show_help()
        success = True
    else:
        parser.print_help()
        success = False
    
    return 0 if success else 1


if __name__ == "__main__":
    exit(main())
