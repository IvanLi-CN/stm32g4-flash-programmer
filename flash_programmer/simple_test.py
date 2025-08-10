#!/usr/bin/env python3
"""
简单的缓冲区测试 - 写入完整的测试数据包
"""

import subprocess
import struct
import time

BUFFER_ADDRESS = 0x20007800
MAGIC_NUMBER = 0xDEADBEEF

def run_command(cmd):
    """运行命令"""
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True, timeout=10)
        return result
    except Exception as e:
        print(f"命令失败: {e}")
        return None

def write_test_packet():
    """写入一个完整的测试数据包"""
    print("=== 写入测试数据包 ===")
    
    # 创建测试数据 - 使用更小的数据包
    test_data = b"Hello, STM32!" + b"\x00" * 16
    test_data = test_data[:32]  # 限制为32字节
    
    # 构建完整的缓冲区内容
    header = struct.pack('<IIII',
                        MAGIC_NUMBER,           # Magic
                        0x00001000,             # Target address (4KB offset)
                        len(test_data),         # Data length
                        1)                      # Status = HAS_DATA
    
    # 填充到2KB
    buffer_content = header + test_data + b'\x00' * (2048 - len(header) - len(test_data))
    
    # 转换为32位值
    values = []
    for i in range(0, len(buffer_content), 4):
        chunk = buffer_content[i:i+4]
        value = struct.unpack('<I', chunk)[0]
        values.append(f"0x{value:08X}")
    
    print(f"数据包信息:")
    print(f"  魔数: 0x{MAGIC_NUMBER:08X}")
    print(f"  地址: 0x00001000")
    print(f"  长度: {len(test_data)} 字节")
    print(f"  状态: 1 (HAS_DATA)")
    print(f"  数据: {test_data[:50]}...")
    
    # 分批写入
    batch_size = 64
    current_address = BUFFER_ADDRESS
    
    for i in range(0, len(values), batch_size):
        batch_values = values[i:i+batch_size]
        
        cmd = [
            "probe-rs", "write",
            "--chip", "STM32G431CB",
            "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",
            "b32",
            f"0x{current_address:08X}"
        ] + batch_values
        
        print(f"写入批次 {i//batch_size + 1}/{(len(values) + batch_size - 1)//batch_size}")
        result = run_command(cmd)
        if result is None:
            print("写入失败!")
            return False
        
        current_address += len(batch_values) * 4
    
    print("✓ 测试数据包写入完成")
    return True

def monitor_status():
    """监控状态变化"""
    print("\n=== 监控状态变化 ===")
    
    last_status = None
    for i in range(100):  # 监控10秒
        cmd = [
            "probe-rs", "read",
            "--chip", "STM32G431CB",
            "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",
            "b32",
            f"0x{BUFFER_ADDRESS + 12:08X}",
            "1"
        ]
        
        result = run_command(cmd)
        if result:
            hex_value = result.stdout.strip().split()[-1]
            status = int(hex_value, 16)
            
            if status != last_status:
                status_names = {0: "IDLE", 1: "HAS_DATA", 2: "PROGRAMMING", 3: "COMPLETE", 4: "ERROR"}
                status_name = status_names.get(status, f"UNKNOWN({status})")
                print(f"状态变化: {last_status} -> {status} ({status_name})")
                last_status = status
                
                if status == 3:  # COMPLETE
                    print("✓ 编程完成!")
                    break
                elif status == 4:  # ERROR
                    print("✗ 编程出错!")
                    break
        
        time.sleep(0.1)
    
    print("监控结束")

def main():
    """主函数"""
    print("STM32G4 Flash Programmer - 简单测试")
    print("=" * 50)
    
    if write_test_packet():
        monitor_status()

if __name__ == "__main__":
    main()
