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

### 基本命令

```bash
# 写入文件 (高速模式)
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file firmware.bin --address 0x0

# 写入文件并验证数据完整性
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file firmware.bin --address 0x0 --verify

# 读取Flash数据
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 read --address 0x0 --size 1024 --output data.bin
```

### 🎨 完整资源管理示例

项目包含完整的W25Q128JV Flash资源管理示例，支持图片、字体、配置等资源的统一管理：

```bash
# 进入示例目录
cd examples/w25q128jv/tools

# 1. 生成开机屏幕 (320x172 RGB565)
python3 svg_to_rgb565.py

# 2. 生成中文字体位图 (文泉驿12px，2094个字符)
python3 font_converter.py

# 3. 生成资源布局配置
python3 resource_manager.py

# 4. 合成完整16MB Flash镜像
python3 flash_composer.py

# 5. 编程到Flash芯片 (16MB，约26分钟)
cd ../../../host-tool
time cargo run --release -- --port /dev/cu.usbmodem412302 write --file ../examples/w25q128jv/w25q128jv_complete.bin --address 0x0
```

### 📋 资源管理功能

| 资源类型 | 地址范围 | 大小 | 描述 |
|---------|---------|------|------|
| 开机屏幕 | 0x00000000-0x0001ADFF | 110KB | 320x172 RGB565位图 |
| 字体数据 | 0x00020000-0x0021FFFF | 2MB | 中文字体位图 (2094字符) |
| UI图形 | 0x00220000-0x0041FFFF | 2MB | UI图标和图形资源 |
| 应用数据 | 0x00420000-0x0071FFFF | 3MB | 应用程序数据存储 |
| 用户配置 | 0x00720000-0x0072FFFF | 64KB | 用户设置和配置 |
| 日志存储 | 0x00730000-0x0074FFFF | 128KB | 系统和错误日志 |
| 固件更新 | 0x00750000-0x007CFFFF | 512KB | 固件更新存储区 |
| 预留空间 | 0x007D0000-0x00FFFFFF | 8.2MB | 未来扩展预留 |

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

### 🏆 实测性能指标 (已优化)

| 文件大小 | 传输时间 | 平均速度 | 成功率 | 测试状态 |
|---------|---------|---------|--------|---------|
| 1KB     | <1秒    | 瞬时完成 | 100%   | ✅ 已验证 |
| 1MB     | 1分32秒  | 11.4 KB/s | 100%  | ✅ 已验证 |
| 16MB    | 26分9秒  | 10.7 KB/s | 100%  | ✅ 已验证 |

### 🚀 性能优化技术

- **批量传输**: 16包批量发送，最大化USB吞吐量
- **双缓冲系统**: 4KB双缓冲，减少内存拷贝开销
- **流式写入**: 无ACK等待，消除往返延迟
- **智能错误处理**: 优雅处理传输异常
- **CRC32校验**: 端到端数据完整性验证

### 📊 性能对比

| 优化阶段 | 传输速度 | 提升幅度 | 关键技术 |
|---------|---------|---------|---------|
| 初始版本 | 4.6 KB/s | 基准 | 基础USB CDC |
| 批量优化 | 8.5 KB/s | +85% | 16包批量传输 |
| 双缓冲优化 | 11.4 KB/s | +148% | 4KB双缓冲 + 流式传输 |

### 💾 内存使用

- **STM32G4 RAM**: 约12KB用于双缓冲和协议栈
- **主机内存**: 约2MB用于文件缓存和进度显示
- **Flash布局**: 支持完整16MB Flash芯片管理

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
