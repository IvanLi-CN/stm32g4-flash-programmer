# STM32G431 外部Flash烧录指南

## 🎯 **烧录流程总结**

### **步骤1：桥接固件烧录** ✅ **已完成**

```bash
cd /Users/ivan/Projects/Ivan/stm32g4-flash-programmer/firmware
cargo build --release
probe-rs run --chip STM32G431CBUx target/thumbv7em-none-eabihf/release/stm32g4-flash-programmer
```

**验证结果：**

- ✅ STM32G431初始化成功
- ✅ 外部Flash (W25Q128JV) 初始化成功
- ✅ USB连接建立
- ✅ 协议处理器启动

### **步骤2：外部Flash内容烧录** 🔄 **进行中**

#### **方法1：使用host-tool命令行工具**

```bash
cd /Users/ivan/Projects/Ivan/stm32g4-flash-programmer/host-tool
./target/release/flash-programmer-tool --port /dev/cu.usbmodem4123302 --timeout 300 write \
  --file ../examples/flash-content-generator/w25q128jv_complete.bin \
  --address 0x000000 --erase --verify
```

**注意事项：**

- 擦除16MB Flash需要很长时间（可能10-30分钟）
- 需要足够的超时时间
- 可能需要多次尝试

#### **注意：Web应用仅用于预览**

⚠️ **重要说明**：`examples/flash-content-generator/web-app/` 目录下的Web应用是**Flash资源预览器**，仅用于可视化查看Flash内容（字体、图片等），**不支持烧录功能**。

如需烧录Flash内容，请使用上述CLI工具方法。

### **步骤3：验证烧录结果**

#### **验证命令：**

```bash
# 检查Flash信息
./target/release/flash-programmer-tool --port /dev/cu.usbmodem4123302 info

# 读取开屏图数据验证
./target/release/flash-programmer-tool --port /dev/cu.usbmodem4123302 read \
  --address 0x000000 --length 110080 --file boot_screen_verify.bin

# 读取16px字体头部验证
./target/release/flash-programmer-tool --port /dev/cu.usbmodem4123302 read \
  --address 0x120000 --length 1024 --file font_16px_header_verify.bin
```

## 📊 **Flash内存映射验证**

根据 `resource_layout.json` 的配置：

| 地址范围 | 内容 | 大小 | 验证方法 |
|---------|------|------|----------|
| 0x00000000 | 开屏图 (320×172 RGB565) | 110KB | 读取前1KB检查非0xFF |
| 0x00020000 | 12px字体数据 | 1MB | 读取字体头部检查字符数量 |
| 0x00120000 | **16px字体数据** ⭐ | 1MB | 读取字体头部检查字符数量 |
| 0x00220000 | UI图形资源 | 2MB | 读取前1KB检查内容 |

## 🔧 **故障排除**

### **常见问题：**

1. **连接超时**
   - 检查USB连接
   - 尝试不同的USB端口
   - 重启桥接固件

2. **擦除超时**
   - 增加超时时间到300秒或更长
   - 分块烧录而不是整体烧录
   - 重新尝试CLI工具烧录

3. **验证失败**
   - 检查文件完整性
   - 重新生成Flash镜像
   - 使用不同的烧录方法

### **USB端口识别：**

```bash
# 查找可用的USB设备
ls /dev/cu.usbmodem*

# 常见端口：
# /dev/cu.usbmodem123456781
# /dev/cu.usbmodem4123202
# /dev/cu.usbmodem4123302
```

## 🚀 **下一步：测试16px字体功能**

烧录完成后，执行以下步骤：

1. **烧录应用固件：**

   ```bash
   cd /Users/ivan/Projects/Ivan/stm32g4-flash-programmer/examples/stm32g431-w25q128jv
   cargo build --release
   probe-rs run --chip STM32G431CBUx target/thumbv7em-none-eabihf/release/flash-content-viewer
   ```

2. **验证功能：**
   - 启动时应显示开屏图（3秒）
   - 主界面应显示16px字体渲染的文本
   - 中英文混合显示应正常工作

3. **测试交互：**
   - BTN1和BTN3按钮应有响应
   - 字体渲染性能应在可接受范围内

## 📝 **当前状态**

- ✅ **桥接固件烧录完成**
- ✅ **Flash镜像文件准备就绪** (16MB)
- 🔄 **正在进行Flash内容烧录**
- ⏳ **等待烧录完成验证**

**预计完成时间：** 10-30分钟（取决于烧录方法和硬件性能）

## 🎯 **成功标准**

烧录成功的标志：

1. Flash信息查询返回正确的JEDEC ID (EF4018)
2. 开屏图数据非全0xFF或全0x00
3. 16px字体头部包含正确的字符数量
4. 应用固件能够成功读取和显示内容
