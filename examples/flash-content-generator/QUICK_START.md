# 🚀 Quick Start Guide - STM32G431CBU6 字体工具

## 📋 前置要求

- Python 3.7+
- PIL/Pillow库：`pip install Pillow`
- STM32开发环境（可选，用于测试）

## ⚡ 5分钟快速开始

### 1. 生成字体文件
```bash
# 进入工具目录
cd flash-programmer-reference/examples/flash-content-generator

# 生成所有字体（推荐）
python font_tools.py generate

# 或者使用完整构建（包含Flash镜像）
python font_tools.py build
```

### 2. 查看生成的字体
```bash
# 查看数字字体信息
python font_tools.py view output/digit_font_24x48.bin --info

# 查看ASCII字体并渲染字符'A'
python font_tools.py view output/ascii_font_16x24.bin --render 33
```

### 3. 验证最终Flash镜像
```bash
# 验证字体是否正确嵌入Flash镜像
python font_tools.py verify pd-sink-128mbit.bin
```

## 🎯 输出文件

执行完成后，您将得到：

```
📁 flash-programmer-reference/examples/flash-content-generator/
├── pd-sink-128mbit.bin              # ✅ 最终Flash镜像 (16MB)
├── output/
│   ├── digit_font_24x48.bin         # ✅ 24×48数字字体
│   └── ascii_font_16x24.bin         # ✅ 16×24 ASCII字体
└── assets/font_output/
    ├── digit_font_24x48.bin         # ✅ 字体副本
    └── ascii_font_16x24.bin         # ✅ 字体副本
```

## 🔧 STM32代码集成

### 1. 添加字体渲染器到您的项目
```rust
// 在main.rs中添加模块
mod font_renderer_digit;
mod font_renderer_ascii;

use font_renderer_digit::FontRendererDigit;
use font_renderer_ascii::FontRendererASCII;
```

### 2. 初始化字体系统
```rust
// 创建字体渲染器
let mut digit_font = FontRendererDigit::new();
let mut ascii_font = FontRendererASCII::new();

// 初始化（需要FlashManager）
digit_font.initialize(&mut flash_manager).await?;
ascii_font.initialize(&mut flash_manager).await?;
```

### 3. 渲染文本到显示缓冲区
```rust
// 准备显示缓冲区
let mut display_buffer = [0u8; 320 * 172]; // 根据您的显示尺寸调整

// 渲染电压数值（大号数字）
digit_font.render_number_string(
    &mut flash_manager,
    "20.5",           // 要显示的数字
    &mut display_buffer,
    320, 172,         // 缓冲区尺寸
    50, 30            // 显示位置 (x, y)
).await?;

// 渲染标签文本（小号文字）
ascii_font.render_text_string(
    &mut flash_manager,
    "Voltage:",       // 要显示的文本
    &mut display_buffer,
    320, 172,         // 缓冲区尺寸
    10, 40            // 显示位置 (x, y)
).await?;
```

## 📊 字体规格速查

| 字体类型 | 尺寸 | 字符集 | 用途 | Flash地址 |
|---------|------|--------|------|-----------|
| 数字字体 | 24×48 | `0-9`, `-`, `.` | 电压/电流/功率显示 | 0x7D0000 |
| ASCII字体 | 16×24 | ASCII 32-126 | 菜单/标签/状态 | 0x7D1000 |

## 🛠️ 常用命令

```bash
# 仅生成数字字体
python font_tools.py generate --digit-only

# 使用特定字体
python font_tools.py generate --font-name "Courier New"

# 查看字符表（前20个字符）
python font_tools.py view output/ascii_font_16x24.bin --table 20

# 渲染特定字符（数字'5'）
python font_tools.py view output/digit_font_24x48.bin --render 5

# 完整验证
python font_tools.py view output/digit_font_24x48.bin --all
```

## 🎨 实际应用示例

### 电源监控界面
```rust
// 显示实时电源参数
let voltage = 20.0f32;
let current = 3.25f32;
let power = voltage * current;

// 格式化数值
let v_str = heapless::String::<16>::from(voltage);
let i_str = heapless::String::<16>::from(current);
let p_str = heapless::String::<16>::from(power);

// 渲染界面
ascii_font.render_text_string(flash_mgr, "Voltage:", buffer, 320, 172, 10, 20).await?;
digit_font.render_number_string(flash_mgr, &v_str, buffer, 320, 172, 120, 10).await?;
ascii_font.render_text_string(flash_mgr, "V", buffer, 320, 172, 250, 20).await?;

ascii_font.render_text_string(flash_mgr, "Current:", buffer, 320, 172, 10, 70).await?;
digit_font.render_number_string(flash_mgr, &i_str, buffer, 320, 172, 120, 60).await?;
ascii_font.render_text_string(flash_mgr, "A", buffer, 320, 172, 250, 70).await?;

ascii_font.render_text_string(flash_mgr, "Power:", buffer, 320, 172, 10, 120).await?;
digit_font.render_number_string(flash_mgr, &p_str, buffer, 320, 172, 120, 110).await?;
ascii_font.render_text_string(flash_mgr, "W", buffer, 320, 172, 250, 120).await?;
```

## 🔍 故障排除

### 问题：字体生成失败
```bash
# 检查Python环境
python --version
pip install Pillow

# 使用默认字体
python font_tools.py generate --font-name "default"
```

### 问题：字符显示不正确
```bash
# 验证字体文件
python font_tools.py view output/digit_font_24x48.bin --validate

# 检查Flash镜像
python font_tools.py verify pd-sink-128mbit.bin
```

### 问题：STM32编译错误
- 确保所有模块都已添加到`main.rs`
- 检查`FlashManager`是否正确初始化
- 验证Flash地址配置

## 📚 更多资源

- **技术文档**: `CUSTOM_FONTS_README.md`
- **使用指南**: `STM32_FONT_USAGE.md`
- **项目总结**: `PROJECT_SUMMARY.md`

## 🎉 完成！

现在您已经拥有了完整的字体系统：
- ✅ 高质量的等宽字体
- ✅ 完整的工具链
- ✅ STM32渲染器
- ✅ 16MB Flash镜像

您的PD-Sink设备现在可以显示专业的数字读数和清晰的文本界面了！

---
*需要帮助？查看详细文档或联系开发团队。*
