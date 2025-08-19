# STM32G4 Flash编程器使用指南

## 🚀 快速开始

### 环境准备

1. **安装Rust工具链**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add thumbv7em-none-eabihf
   ```

2. **安装调试工具**

   ```bash
   cargo install probe-rs --features cli
   ```

3. **Python环境** (用于资源生成)

   ```bash
   pip install pillow numpy
   ```

### 硬件连接

#### STM32G431CBU6 ↔ W25Q128JV 连接图

```text
STM32G431CBU6          W25Q128JV Flash
┌─────────────┐       ┌─────────────┐
│ PB12 (NSS)  │────── │ CS          │
│ PB13 (SCK)  │────── │ CLK         │
│ PB14 (MISO) │────── │ DO          │
│ PB15 (MOSI) │────── │ DI          │
│ 3.3V        │────── │ VCC         │
│ GND         │────── │ GND         │
└─────────────┘       └─────────────┘
```

## 🔧 编译和部署

### 1. 编译固件

```bash
cd firmware
cargo build --release --target thumbv7em-none-eabihf
```

### 2. 烧录固件到STM32G4

```bash
probe-rs download --chip STM32G431CBUx target/thumbv7em-none-eabihf/release/stm32g4-flash-programmer
```

### 3. 编译主机工具

```bash
cd host-tool
cargo build --release
```

## 📊 性能测试

### 基准测试

```bash
# 创建测试文件
dd if=/dev/zero of=test_1mb.bin bs=1024 count=1024

# 高速写入测试
time ./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file test_1mb.bin --address 0x0

# 带验证的写入测试
time ./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file test_1mb.bin --address 0x0 --verify
```

### 预期性能指标

| 文件大小 | 传输时间 | 平均速度 | 备注 |
|---------|---------|---------|------|
| 1KB     | <1秒    | 瞬时    | 小文件优化 |
| 1MB     | ~1.5分钟 | 11.4 KB/s | 批量传输 |
| 16MB    | ~26分钟  | 10.7 KB/s | 大文件稳定 |

## 🎨 资源管理工作流

### 完整资源生成流程

```bash
cd examples/w25q128jv/tools

# Step 1: 生成开机屏幕
python3 svg_to_rgb565.py
# 输出: ../assets/boot_screen_320x172.bin (110KB)

# Step 2: 生成字体位图
python3 font_converter.py
# 输出: ../assets/font_output/font_bitmap.bin (53KB)
# 包含2094个字符 (ASCII + 中文)

# Step 3: 生成内存布局
python3 resource_manager.py
# 输出: ../assets/memory_map.txt
# 输出: ../assets/resource_layout.json

# Step 4: 合成完整Flash镜像
python3 flash_composer.py
# 输出: ../w25q128jv_complete.bin (16MB)
```

### 编程完整固件

```bash
cd ../../../host-tool
time cargo run --release -- --port /dev/cu.usbmodem412302 write --file ../examples/w25q128jv/w25q128jv_complete.bin --address 0x0
```

## 🛠️ 高级功能

### 数据完整性验证

```bash
# CRC32校验 (推荐)
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file data.bin --address 0x0 --verify

# 读回验证
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 read --address 0x0 --size 1048576 --output readback.bin
diff data.bin readback.bin
```

### 批量操作

```bash
# 批量写入多个文件
for file in *.bin; do
    echo "Programming $file..."
    ./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file "$file" --address 0x0
done
```

### 性能调优

```bash
# 环境变量调优
export RUST_LOG=info                    # 启用详细日志
export USB_TIMEOUT=30000                # 增加USB超时时间
export BATCH_SIZE=16                    # 调整批量大小

# 运行优化版本
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file large_file.bin --address 0x0
```

## 🐛 故障排除

### 常见问题

#### 1. 设备连接问题

```bash
# 检查设备
ls /dev/cu.usbmodem*

# 权限问题 (Linux)
sudo chmod 666 /dev/ttyACM0

# macOS权限
sudo dscl . append /Groups/wheel GroupMembership $(whoami)
```

#### 2. 传输错误

```bash
# 检查USB连接稳定性
./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file small_test.bin --address 0x0

# 降低传输速度
export BATCH_SIZE=8
export USB_DELAY=10
```

#### 3. Flash操作失败

```bash
# 检查SPI连接
# 使用万用表测试连接
# 检查电源稳定性 (3.3V ±0.1V)
```

### 调试模式

```bash
# 固件调试
cd firmware
probe-rs attach --chip STM32G431CBUx

# 主机工具调试
RUST_LOG=debug ./target/release/stm32g4-flash-tool --port /dev/cu.usbmodem412302 write --file test.bin --address 0x0
```

## 📈 性能优化技巧

### 1. 系统优化

```bash
# macOS: 禁用USB节能
sudo pmset -a usbwakeup 0

# Linux: 调整USB缓冲区
echo 16384 | sudo tee /sys/module/usbcore/parameters/usbfs_memory_mb
```

### 2. 传输优化

- 使用高质量USB线缆 (≤1米)
- 避免USB集线器
- 关闭不必要的USB设备
- 使用SSD存储测试文件

### 3. 固件优化

- 双缓冲系统已启用
- 批量传输已优化
- 中断优先级已调整

## 🔍 技术细节

### 通信协议

```text
数据包格式:
┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Magic   │ Command │ Length  │ Address │ Data    │ CRC16   │
│ (2B)    │ (1B)    │ (4B)    │ (4B)    │ (nB)    │ (2B)    │
└─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘
```

### 性能分析

- **USB CDC理论极限**: ~12 KB/s
- **实际达到**: 10.7 KB/s (89%效率)
- **批量优化**: 16包/批次
- **缓冲优化**: 4KB双缓冲

---

**📝 文档版本**: v2.0
**🐾 作者**: 鸣濑白羽 (猫娘心羽)
**📅 更新时间**: 2024年
