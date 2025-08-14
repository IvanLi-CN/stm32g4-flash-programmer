# STM32G4 Flash编程器技术架构

## 🏗️ 系统架构概览

```
┌─────────────────┐    USB CDC    ┌─────────────────┐    SPI    ┌─────────────────┐
│                 │◄─────────────►│                 │◄─────────►│                 │
│   Host Tool     │               │  STM32G4 MCU    │           │  W25Q128JV      │
│   (Rust/Tokio)  │               │  (Embassy)      │           │  Flash Memory   │
│                 │               │                 │           │                 │
└─────────────────┘               └─────────────────┘           └─────────────────┘
```

## 🔧 核心组件

### 1. 主机端工具 (Host Tool)

#### 技术栈
- **语言**: Rust 1.70+
- **异步运行时**: Tokio
- **串口通信**: tokio-serial
- **进度显示**: indicatif
- **数据完整性**: CRC32, SHA256

#### 核心模块

```rust
host-tool/
├── src/
│   ├── main.rs           // 命令行界面和参数解析
│   ├── serial.rs         // USB CDC串口通信层
│   ├── commands.rs       // Flash操作命令实现
│   └── protocol.rs       // 通信协议处理
└── Cargo.toml
```

#### 关键特性
- **高性能传输**: 批量数据包 + 双缓冲
- **错误恢复**: 自动重试和优雅降级
- **进度监控**: 实时速度和ETA显示
- **数据验证**: 多层次完整性保证

### 2. STM32G4固件 (Firmware)

#### 技术栈
- **语言**: Rust (no_std)
- **HAL**: Embassy STM32
- **USB栈**: Embassy USB
- **SPI驱动**: Embassy SPI

#### 核心模块

```rust
firmware/
├── src/
│   ├── main.rs           // 主程序和任务调度
│   ├── usb_handler.rs    // USB CDC处理
│   ├── flash_driver.rs   // SPI Flash驱动
│   ├── protocol.rs       // 协议解析
│   └── buffer.rs         // 双缓冲管理
├── memory.x              // 内存布局
└── Cargo.toml
```

#### 内存管理
```
STM32G431CBU6 内存布局:
┌─────────────────┐ 0x20000000 + 32KB
│   Stack         │
├─────────────────┤
│   Heap          │
├─────────────────┤
│   USB Buffers   │ 4KB × 2 (双缓冲)
├─────────────────┤
│   Protocol      │ 2KB
├─────────────────┤
│   Static Data   │
└─────────────────┘ 0x20000000
```

### 3. 通信协议 (Protocol)

#### 数据包格式

```
命令包 (Host → MCU):
┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Magic   │ Command │ Seq     │ Length  │ Address │ Data    │
│ 0xABCD  │ u8      │ u16     │ u32     │ u32     │ [u8]    │
│ (2B)    │ (1B)    │ (2B)    │ (4B)    │ (4B)    │ (nB)    │
└─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘

响应包 (MCU → Host):
┌─────────┬─────────┬─────────┬─────────┬─────────┐
│ Magic   │ Status  │ Seq     │ Length  │ Data    │
│ 0xDCBA  │ u8      │ u16     │ u32     │ [u8]    │
│ (2B)    │ (1B)    │ (2B)    │ (4B)    │ (nB)    │
└─────────┴─────────┴─────────┴─────────┴─────────┘
```

#### 命令集

| 命令 | 值 | 描述 | 参数 |
|------|----|----- |------|
| Info | 0x01 | 获取Flash信息 | 无 |
| Erase | 0x02 | 擦除Flash区域 | address, size |
| Write | 0x03 | 写入数据 | address, data |
| Read | 0x04 | 读取数据 | address, size |
| Verify | 0x05 | 验证数据 | address, data |
| StreamWrite | 0x08 | 流式写入 | address, data |
| VerifyCRC | 0x09 | CRC校验 | address, crc32 |

## ⚡ 性能优化架构

### 1. 批量传输系统

```rust
// 主机端批量发送
const BATCH_SIZE: usize = 16;
let mut batch = Vec::with_capacity(BATCH_SIZE);

for chunk in data.chunks(MAX_PAYLOAD_SIZE) {
    batch.push(create_packet(chunk));
    
    if batch.len() == BATCH_SIZE {
        send_batch(&mut batch).await?;
        batch.clear();
    }
}
```

### 2. 双缓冲系统

```rust
// 固件端双缓冲
static mut USB_RX_BUFFER_1: [u8; 4096] = [0; 4096];
static mut USB_RX_BUFFER_2: [u8; 4096] = [0; 4096];
static mut CURRENT_BUFFER: AtomicBool = AtomicBool::new(false);

fn get_current_buffer() -> &'static mut [u8; 4096] {
    unsafe {
        if CURRENT_BUFFER.load(Ordering::Relaxed) {
            &mut USB_RX_BUFFER_2
        } else {
            &mut USB_RX_BUFFER_1
        }
    }
}

fn switch_buffer() {
    CURRENT_BUFFER.fetch_xor(true, Ordering::Relaxed);
}
```

### 3. 流水线处理

```
时间轴:
t0: [USB接收] [     空闲     ] [     空闲     ]
t1: [USB接收] [SPI写入Buffer1] [     空闲     ]
t2: [USB接收] [SPI写入Buffer1] [Flash编程     ]
t3: [USB接收] [SPI写入Buffer2] [Flash编程     ]
t4: [USB接收] [SPI写入Buffer2] [Flash编程     ]
```

## 🛡️ 数据完整性架构

### 1. 多层次验证

```
数据流验证层次:
┌─────────────────┐
│ 应用层SHA256    │ ← 端到端文件完整性
├─────────────────┤
│ 协议层CRC32     │ ← 传输过程完整性
├─────────────────┤
│ 传输层序列号    │ ← 数据包顺序验证
├─────────────────┤
│ 物理层USB CRC   │ ← 硬件层错误检测
└─────────────────┘
```

### 2. 错误恢复机制

```rust
// 智能重试策略
async fn send_with_retry<T>(
    operation: impl Fn() -> Future<Output = Result<T>>,
    max_retries: usize
) -> Result<T> {
    let mut retries = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(100 * retries)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 🎨 资源管理架构

### 1. Flash内存布局

```
W25Q128JV (16MB) 内存映射:
┌─────────────────┐ 0x00000000
│ Boot Screen     │ 110KB  (开机画面)
├─────────────────┤ 0x00020000
│ Font Bitmap     │ 2MB    (字体数据)
├─────────────────┤ 0x00220000
│ UI Graphics     │ 2MB    (UI图形)
├─────────────────┤ 0x00420000
│ App Data        │ 3MB    (应用数据)
├─────────────────┤ 0x00720000
│ User Config     │ 64KB   (用户配置)
├─────────────────┤ 0x00730000
│ Log Storage     │ 128KB  (日志存储)
├─────────────────┤ 0x00750000
│ Firmware Update │ 512KB  (固件更新)
├─────────────────┤ 0x007D0000
│ Reserved        │ 8.2MB  (预留空间)
└─────────────────┘ 0x00FFFFFF
```

### 2. 资源生成工具链

```python
# 工具链架构
tools/
├── svg_to_rgb565.py      # SVG → RGB565位图转换
├── font_converter.py     # TTF → 位图字体转换
├── resource_manager.py   # 内存布局管理
└── flash_composer.py     # 完整镜像合成
```

## 📊 性能分析

### 1. 传输性能模型

```
理论最大速度计算:
USB Full Speed = 12 Mbps
CDC ACM开销 ≈ 15%
实际可用带宽 ≈ 10.2 Mbps ≈ 1.275 MB/s

实测性能:
批量优化前: 4.6 KB/s  (0.36% 效率)
批量优化后: 11.4 KB/s (0.89% 效率)
```

### 2. 内存使用分析

```
STM32G4 RAM使用:
├── USB缓冲区: 8KB  (双缓冲 4KB×2)
├── 协议栈:   2KB  (数据包处理)
├── SPI缓冲:  1KB  (Flash操作)
├── 系统栈:   2KB  (Embassy运行时)
└── 其他:     1KB  (变量和常量)
总计:        14KB / 32KB (44%使用率)
```

### 3. 延迟分析

```
端到端延迟组成:
┌─────────────────┐
│ 应用处理: 1ms   │
├─────────────────┤
│ USB传输: 8ms    │ ← 主要瓶颈
├─────────────────┤
│ 协议解析: 0.1ms │
├─────────────────┤
│ SPI传输: 2ms    │
├─────────────────┤
│ Flash编程: 5ms  │
└─────────────────┘
总延迟: ~16ms/包
```

## 🔮 扩展性设计

### 1. 支持多种Flash芯片

```rust
trait FlashDriver {
    async fn read(&mut self, addr: u32, data: &mut [u8]) -> Result<()>;
    async fn write(&mut self, addr: u32, data: &[u8]) -> Result<()>;
    async fn erase(&mut self, addr: u32, size: u32) -> Result<()>;
    fn info(&self) -> FlashInfo;
}

// 具体实现
impl FlashDriver for W25Q128JV { ... }
impl FlashDriver for W25Q256JV { ... }
impl FlashDriver for AT25SF128A { ... }
```

### 2. 协议版本兼容

```rust
#[derive(Debug, Clone)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

// 向后兼容性检查
fn is_compatible(host_version: &ProtocolVersion, device_version: &ProtocolVersion) -> bool {
    host_version.major == device_version.major &&
    host_version.minor >= device_version.minor
}
```

---

**📝 文档版本**: v2.0  
**🐾 作者**: 鸣濑白羽 (猫娘心羽)  
**📅 更新时间**: 2024年
