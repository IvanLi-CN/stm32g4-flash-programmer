# STM32G4 Flash Programmer

一个基于Embassy框架的STM32G4 USB CDC Flash编程器，用于通过USB虚拟串口对外部SPI Flash (W25Q128)进行读写操作。

## 🚀 特性

- **USB CDC通信**: 无需驱动，即插即用的虚拟串口
- **异步处理**: 基于Embassy框架的高效异步操作
- **大文件支持**: 支持16MB文件的分块传输
- **完整性保证**: CRC校验确保数据传输可靠性
- **进度显示**: 实时显示传输进度和速度
- **多种操作**: 支持读取、写入、擦除、验证操作

## 📋 项目结构

```
stm32g4-flash-programmer/
├── firmware/           # STM32G4固件 (Embassy + USB CDC)
│   ├── src/
│   │   ├── main.rs
│   │   ├── flash_driver.rs
│   │   ├── protocol_handler.rs
│   │   └── usb_cdc.rs
│   ├── Cargo.toml
│   ├── memory.x
│   └── Embed.toml
├── host-tool/          # PC端工具 (Rust CLI)
│   ├── src/
│   │   ├── main.rs
│   │   ├── serial.rs
│   │   └── commands.rs
│   └── Cargo.toml
├── protocol/           # 共享通信协议
│   ├── src/lib.rs
│   └── Cargo.toml
└── README.md
```

## 🔌 硬件连接

### STM32G431CBU6 ↔ W25Q128 SPI Flash

| W25Q128 引脚 | STM32 引脚 | 功能 | 物理引脚 |
|-------------|-----------|------|---------|
| CS          | PB12      | SPI2_NSS | Pin 25 |
| CLK         | PB13      | SPI2_SCK | Pin 26 |
| DI (MOSI)   | PB15      | SPI2_MOSI| Pin 28 |
| DO (MISO)   | PB14      | SPI2_MISO| Pin 27 |
| VCC         | 3.3V      | 电源     | - |
| GND         | GND       | 地线     | - |

### USB连接

| 功能 | STM32 引脚 |
|------|-----------|
| USB_DP | PA12 |
| USB_DM | PA11 |

## 🚀 快速开始

### 一键构建和测试

```bash
# 1. 构建PC端工具
cargo build --release -p flash-programmer-tool

# 2. 构建固件 (需要先连接STM32G4开发板)
./build_firmware.sh

# 3. 运行测试 (需要连接硬件和Flash芯片)
./test_example.sh
```

## 🛠️ 详细编译步骤

### 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 ARM Cortex-M4 目标
rustup target add thumbv7em-none-eabihf

# 安装 probe-rs
cargo install probe-rs --features cli
```

### 编译固件

```bash
cd firmware
cargo build --release --target thumbv7em-none-eabihf
```

### 烧录固件

```bash
cd firmware
probe-rs download --chip STM32G431CBUx target/thumbv7em-none-eabihf/release/stm32g4-flash-programmer
```

### 编译PC端工具

```bash
cargo build --release -p flash-programmer-tool
```

## 📡 使用方法

### 获取Flash信息

```bash
./target/release/flash-programmer-tool --port /dev/ttyACM0 info
```

### 擦除Flash

```bash
# 擦除从地址0开始的64KB
./target/release/flash-programmer-tool --port /dev/ttyACM0 erase -a 0x0 -s 0x10000
```

### 写入文件

```bash
# 写入文件到Flash，自动擦除并验证
./target/release/flash-programmer-tool --port /dev/ttyACM0 write -f firmware.bin -a 0x0 --erase --verify
```

### 读取Flash到文件

```bash
# 读取16MB数据到文件
./target/release/flash-programmer-tool --port /dev/ttyACM0 read -f readback.bin -a 0x0 -s 0x1000000
```

### 验证文件

```bash
# 验证文件与Flash内容是否一致
./target/release/flash-programmer-tool --port /dev/ttyACM0 verify -f firmware.bin -a 0x0
```

## 🔧 通信协议

### 数据包格式

```
命令包: [MAGIC:2][CMD:1][LEN:4][ADDR:4][DATA:LEN][CRC:2]
响应包: [MAGIC:2][STATUS:1][LEN:4][DATA:LEN][CRC:2]
```

### 支持的命令

- **INFO (0x01)**: 获取Flash信息
- **ERASE (0x02)**: 擦除Flash区域
- **WRITE (0x03)**: 写入数据到Flash
- **READ (0x04)**: 从Flash读取数据
- **VERIFY (0x05)**: 验证数据完整性

### 状态码

- **SUCCESS (0x00)**: 操作成功
- **INVALID_COMMAND (0x01)**: 无效命令
- **INVALID_ADDRESS (0x02)**: 无效地址
- **FLASH_ERROR (0x03)**: Flash操作失败
- **CRC_ERROR (0x04)**: CRC校验失败
- **BUFFER_OVERFLOW (0x05)**: 缓冲区溢出
- **TIMEOUT (0x06)**: 操作超时

## ⚡ 性能数据

- **传输速度**: 约500KB/s - 1MB/s (取决于USB和Flash性能)
- **16MB传输时间**: 约20-30秒
- **内存使用**: STM32 RAM约8KB用于缓冲

## 🐛 故障排除

### 常见问题

1. **设备未识别**
   - 检查USB连接
   - 确认固件已正确烧录
   - 在Linux下检查设备权限

2. **Flash操作失败**
   - 检查SPI连接
   - 确认Flash芯片型号
   - 检查电源稳定性

3. **传输错误**
   - 检查USB线缆质量
   - 尝试降低传输块大小
   - 检查系统USB驱动

### 调试模式

固件包含defmt日志输出，可以通过probe-rs查看：

```bash
cd firmware
probe-rs attach --chip STM32G431CBUx
```

## 📄 许可证

本项目采用 MIT 许可证。

## 🤝 贡献

欢迎提交Issue和Pull Request！

---

**🐾 Made with ❤️ by 鸣濑白羽 (猫娘心羽)**
