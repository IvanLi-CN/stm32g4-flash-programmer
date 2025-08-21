#!/usr/bin/env python3
"""
简单的HTTP服务器启动脚本
用于本地测试STM32G431CBU6 Flash资源预览器

Usage:
    python start_server.py [port]

默认端口: 8080
"""

import http.server
import socketserver
import os
import sys
import webbrowser
from pathlib import Path

def main():
    # 默认端口
    port = 8080
    
    # 如果提供了端口参数
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print(f"❌ 无效的端口号: {sys.argv[1]}")
            sys.exit(1)
    
    # 确保在正确的目录中
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # 检查必要文件
    required_files = [
        'index.html',
        'script.js',
        'pd-sink-128mbit.bin'
    ]
    
    missing_files = []
    for file in required_files:
        if not Path(file).exists():
            missing_files.append(file)
    
    if missing_files:
        print("❌ 缺少必要文件:")
        for file in missing_files:
            print(f"   - {file}")
        print("\n请确保所有文件都在web-app目录中")
        sys.exit(1)
    
    # 检查固件文件大小
    firmware_file = Path('pd-sink-128mbit.bin')
    if firmware_file.exists():
        size_mb = firmware_file.stat().st_size / (1024 * 1024)
        print(f"✅ 固件文件: {firmware_file.name} ({size_mb:.1f} MB)")
    
    # 创建HTTP服务器
    handler = http.server.SimpleHTTPRequestHandler
    
    # 添加CORS头部支持
    class CORSRequestHandler(handler):
        def end_headers(self):
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
            self.send_header('Access-Control-Allow-Headers', '*')
            super().end_headers()
    
    try:
        with socketserver.TCPServer(("", port), CORSRequestHandler) as httpd:
            print(f"🚀 STM32G431CBU6 Flash资源预览器")
            print(f"📡 服务器启动在端口: {port}")
            print(f"🌐 访问地址: http://localhost:{port}")
            print(f"📁 服务目录: {script_dir}")
            print(f"🔧 测试页面: http://localhost:{port}/test.html")
            print("\n按 Ctrl+C 停止服务器")
            
            # 自动打开浏览器
            try:
                webbrowser.open(f'http://localhost:{port}')
                print("🌐 已自动打开浏览器")
            except:
                print("⚠️ 无法自动打开浏览器，请手动访问上述地址")
            
            # 启动服务器
            httpd.serve_forever()
            
    except OSError as e:
        if e.errno == 48:  # Address already in use
            print(f"❌ 端口 {port} 已被占用")
            print(f"💡 尝试使用其他端口: python {sys.argv[0]} {port + 1}")
        else:
            print(f"❌ 启动服务器失败: {e}")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n👋 服务器已停止")

if __name__ == "__main__":
    main()
