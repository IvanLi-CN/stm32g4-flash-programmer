#!/usr/bin/env python3
"""
STM32G4 Flash Programmer - Memory Buffer Protocol
使用 probe-rs 通过内存缓冲区协议烧录 16MB 数据到外部 W25Q128 Flash
"""

import subprocess
import sys
import time
import os
import struct
from pathlib import Path

# Buffer protocol constants
MAGIC_NUMBER = 0xDEADBEEF
BUFFER_ADDRESS = 0x20007800  # From memory.x
BUFFER_SIZE = 2048
HEADER_SIZE = 16
DATA_SIZE = 256  # Reduce from 2032 to 256 bytes for stability

# Status values
STATUS_IDLE = 0
STATUS_HAS_DATA = 1
STATUS_PROGRAMMING = 2
STATUS_COMPLETE = 3
STATUS_ERROR = 4

def run_command(cmd, description="", capture_output=True, timeout=30):
    """运行命令并处理错误"""
    print(f"执行: {description}")
    print(f"命令: {' '.join(cmd)}")
    try:
        if capture_output:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True, timeout=timeout)
            if result.stdout:
                print(result.stdout)
            return result
        else:
            process = subprocess.Popen(cmd)
            return process
    except subprocess.CalledProcessError as e:
        print(f"错误: {description} 失败")
        print(f"命令: {' '.join(cmd)}")
        if hasattr(e, 'stderr') and e.stderr:
            print(f"错误输出: {e.stderr}")
        return None
    except subprocess.TimeoutExpired:
        print(f"错误: {description} 超时")
        return None

def check_prerequisites():
    """检查必要的工具和文件"""
    print("=== 检查必要条件 ===")
    
    # 检查 probe-rs
    if subprocess.run(["which", "probe-rs"], capture_output=True).returncode != 0:
        print("错误: 未找到 probe-rs")
        print("请安装: cargo install probe-rs --features cli")
        return False
    print("✓ 找到 probe-rs")
    
    # 检查固件文件
    firmware_file = "../examples/w25q128jv/w25q128jv_complete.bin"
    if not os.path.exists(firmware_file):
        print(f"错误: 找不到固件文件: {firmware_file}")
        return False
    
    file_size = os.path.getsize(firmware_file)
    print(f"✓ 找到固件文件: {firmware_file}")
    print(f"  文件大小: {file_size:,} 字节 ({file_size // (1024*1024)} MB)")
    
    return True, firmware_file, file_size

def build_and_flash_programmer():
    """编译并烧录 Flash Programmer"""
    print("\n=== 编译 Flash Programmer ===")
    if not run_command(["cargo", "build", "--release"], "编译 Flash Programmer"):
        return False

    print("\n=== 烧录 Flash Programmer 到 STM32 ===")
    print("提示: 如果连接失败，请检查:")
    print("  1. STM32 开发板是否正确连接")
    print("  2. 是否需要按住 RESET 按钮")
    print("  3. 调试器是否正常工作")

    # 尝试多种连接方式
    download_commands = [
        # 标准连接
        [
            "probe-rs", "download",
            "--chip", "STM32G431CB",
            "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",  # 完整的调试器标识
            "target/thumbv7em-none-eabihf/release/flash_programmer"
        ],
        # 复位下连接
        [
            "probe-rs", "download",
            "--chip", "STM32G431CB",
            "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",
            "--connect-under-reset",
            "target/thumbv7em-none-eabihf/release/flash_programmer"
        ]
    ]

    for i, cmd in enumerate(download_commands):
        print(f"\n尝试连接方式 {i+1}...")
        result = run_command(cmd, f"烧录 STM32 程序 (方式 {i+1})", timeout=60)
        if result is not None:
            return True
        print(f"方式 {i+1} 失败，尝试下一种方式...")

    print("\n所有连接方式都失败了")
    return False

def read_buffer_status():
    """读取缓冲区状态"""
    cmd = [
        "probe-rs", "read",
        "--chip", "STM32G431CB",
        "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",
        "b32",  # 32位宽度
        f"0x{BUFFER_ADDRESS + 12:08X}",  # Status offset
        "1"  # 1 个32位字 = 4 bytes
    ]

    result = run_command(cmd, "读取缓冲区状态", timeout=5)
    if result is None:
        return None

    try:
        # Parse hex output from probe-rs
        # probe-rs returns 32-bit words in hex format like "124a47d5"
        hex_output = result.stdout.strip()
        print(f"原始输出: {hex_output}")

        # Extract the hex value (remove any whitespace)
        hex_value = hex_output.split()[-1]  # Get the last hex value
        print(f"十六进制值: {hex_value}")

        # Convert hex string to integer (big-endian from probe-rs)
        status = int(hex_value, 16)
        print(f"状态值: {status} (0x{status:08X})")

        return status
    except (ValueError, IndexError) as e:
        print(f"解析状态失败: {e}")
        print(f"原始输出: '{result.stdout}'")
        return None

def write_buffer_data(address, data):
    """写入数据到缓冲区"""
    if len(data) > DATA_SIZE:
        raise ValueError(f"数据太大: {len(data)} > {DATA_SIZE}")

    # 构建缓冲区内容
    header = struct.pack('<IIII',
                        MAGIC_NUMBER,      # Magic
                        address,           # Target address
                        len(data),         # Data length
                        STATUS_HAS_DATA)   # Status

    # 填充数据到完整缓冲区大小
    buffer_content = header + data + b'\x00' * (BUFFER_SIZE - len(header) - len(data))

    # 将缓冲区内容转换为32位值列表
    values = []
    for i in range(0, len(buffer_content), 4):
        chunk = buffer_content[i:i+4]
        if len(chunk) < 4:
            chunk += b'\x00' * (4 - len(chunk))  # 填充到4字节
        value = struct.unpack('<I', chunk)[0]  # Little-endian u32
        values.append(f"0x{value:08X}")

    # 使用 probe-rs 写入内存 (分批写入以避免命令行过长)
    batch_size = 64  # 每批写入64个32位值 = 256字节
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

        result = run_command(cmd, f"写入批次 {i//batch_size + 1} ({len(batch_values)} 个值)", timeout=15)
        if result is None:
            return False

        current_address += len(batch_values) * 4

    return True

def wait_for_status(expected_status, timeout=30):
    """等待特定状态"""
    start_time = time.time()
    last_status = None

    # 先等待一小段时间让STM32处理
    time.sleep(0.5)

    while time.time() - start_time < timeout:
        status = read_buffer_status()

        if status != last_status:
            status_names = {0: "IDLE", 1: "HAS_DATA", 2: "PROGRAMMING", 3: "COMPLETE", 4: "ERROR"}
            status_name = status_names.get(status, f"UNKNOWN({status})")
            print(f"状态变化: {last_status} -> {status} ({status_name})")
            last_status = status

        if status == expected_status:
            return True
        elif status == STATUS_ERROR:
            print("STM32 报告编程错误")
            return False

        time.sleep(0.2)  # 200ms polling interval

    print(f"等待状态 {expected_status} 超时")
    return False

def verify_flash_data(firmware_file, file_size):
    """验证Flash中的数据"""
    print(f"\n=== 验证 Flash 数据 ===")

    # 创建验证命令数据包
    verify_cmd = struct.pack('<IIII',
                            0xCAFEBABE,        # 验证命令魔数
                            0x00000000,        # 起始地址
                            file_size,         # 验证长度
                            5)                 # 状态 = VERIFY_REQUEST

    # 填充到完整缓冲区
    buffer_content = verify_cmd + b'\x00' * (BUFFER_SIZE - len(verify_cmd))

    # 转换为32位值并写入
    values = []
    for i in range(0, len(buffer_content), 4):
        chunk = buffer_content[i:i+4]
        value = struct.unpack('<I', chunk)[0]
        values.append(f"0x{value:08X}")

    # 分批写入验证命令
    batch_size = 64
    current_address = BUFFER_ADDRESS

    print("发送验证命令...")
    for i in range(0, len(values), batch_size):
        batch_values = values[i:i+batch_size]

        cmd = [
            "probe-rs", "write",
            "--chip", "STM32G431CB",
            "--probe", "0d28:0204:1B5EE3417B8AC52C29960D1C660DD59A",
            "b32",
            f"0x{current_address:08X}"
        ] + batch_values

        result = run_command(cmd, f"写入验证命令批次 {i//batch_size + 1}", timeout=15)
        if result is None:
            return False

        current_address += len(batch_values) * 4

    # 等待验证完成
    print("等待验证完成...")
    if not wait_for_status(6, timeout=300):  # STATUS_VERIFY_COMPLETE = 6
        print("验证超时或失败")
        return False

    print("✓ Flash 数据验证成功!")
    return True

def program_flash_file(firmware_file, file_size):
    """使用缓冲区协议烧录文件"""
    print(f"\n=== 开始烧录 {file_size:,} 字节数据 ===")

    with open(firmware_file, "rb") as f:
        total_bytes = 0
        chunk_count = 0
        start_time = time.time()

        while total_bytes < file_size:
            # 读取数据块
            remaining = file_size - total_bytes
            chunk_size = min(remaining, DATA_SIZE)
            chunk_data = f.read(chunk_size)

            if not chunk_data:
                break

            chunk_count += 1
            target_address = total_bytes

            print(f"\n--- 块 {chunk_count}: 地址 0x{target_address:08X}, 大小 {len(chunk_data)} 字节 ---")

            # 写入数据到缓冲区
            if not write_buffer_data(target_address, chunk_data):
                print("写入缓冲区失败")
                return False

            # 等待 STM32 完成编程
            print("等待 STM32 完成编程...")
            if not wait_for_status(STATUS_COMPLETE, timeout=60):
                print("编程超时或失败")
                return False

            total_bytes += len(chunk_data)
            elapsed = time.time() - start_time
            speed = total_bytes / elapsed if elapsed > 0 else 0
            progress = (total_bytes / file_size) * 100

            print(f"✓ 块 {chunk_count} 编程完成")
            print(f"进度: {progress:.1f}% ({total_bytes:,}/{file_size:,} 字节)")
            print(f"速度: {speed:.0f} 字节/秒")

            # 清理缓冲区状态，准备下一块
            time.sleep(0.1)  # 短暂延迟确保状态稳定

    elapsed = time.time() - start_time
    avg_speed = total_bytes / elapsed if elapsed > 0 else 0

    print(f"\n✓ 烧录完成!")
    print(f"总计: {total_bytes:,} 字节")
    print(f"用时: {elapsed:.1f} 秒")
    print(f"平均速度: {avg_speed:.0f} 字节/秒")

    return True

def main():
    """主函数"""
    print("=== STM32G4 Flash Programmer - 内存缓冲区协议 ===")
    print("使用 probe-rs 内存写入 + STM32 自动烧录")
    print()
    
    # 检查必要条件
    result = check_prerequisites()
    if isinstance(result, tuple):
        success, firmware_file, file_size = result
        if not success:
            sys.exit(1)
    else:
        sys.exit(1)
    
    # 跳过编译和烧录 - 假设STM32程序已经在运行
    print("\n=== 跳过 STM32 程序烧录 (假设已在运行) ===")
    print("注意: 请确保 STM32 程序已经通过 probe-rs run 启动并运行")

    # 短暂等待确保系统稳定
    time.sleep(1)
    
    # 检查初始状态
    print("检查缓冲区初始状态...")
    initial_status = read_buffer_status()
    if initial_status is None:
        print("无法读取缓冲区状态")
        sys.exit(1)
    
    print(f"初始状态: {initial_status}")
    
    # 开始烧录
    if program_flash_file(firmware_file, file_size):
        print("\n🎉 Flash 烧录成功完成!")

        # 验证烧录的数据
        print("\n=== 开始数据验证 ===")
        if verify_flash_data(firmware_file, file_size):
            print("\n🎉 Flash 数据验证成功! 烧录完全正确!")
        else:
            print("\n❌ Flash 数据验证失败! 烧录可能有问题!")
            sys.exit(1)
    else:
        print("\n❌ Flash 烧录失败")
        sys.exit(1)

if __name__ == "__main__":
    main()
