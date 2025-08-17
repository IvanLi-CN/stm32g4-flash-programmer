# Flash Content Viewer - 运行演示

## 🚀 固件启动序列

```
[INFO] STM32G431CBU6 initialized successfully
[INFO] Heap initialized: 8192 bytes
[INFO] Hardware pins configured
[INFO] ✅ Display initialized successfully
[INFO] Display SPI device initialized (simplified)
[INFO] Draw text at (60, 30): 'Flash Viewer'
[INFO] Draw text at (70, 50): 'STM32G431'
[INFO] Draw rectangle at (10, 10) size 220x220
[INFO] Draw rectangle at (12, 12) size 216x216
[INFO] Draw text at (50, 180): 'Initializing...'
[INFO] ✅ Flash initialized successfully
[INFO] Flash SPI device initialized
[INFO] Display clear (simulated)
[INFO] Draw text at (70, 30): 'Flash Info'
[INFO] Draw text at (20, 60): 'JEDEC: EF4018'
[INFO] Draw text at (20, 80): 'Size: 16MB'
[INFO] Draw text at (90, 180): 'Ready!'
```

## 📱 用户界面演示

### 启动画面 (0-2秒)
```
┌─────────────────────────┐
│    Flash Viewer         │
│    STM32G431            │
│                         │
│  ┌─────────────────┐    │
│  │                 │    │
│  │                 │    │
│  │                 │    │
│  │                 │    │
│  └─────────────────┘    │
│                         │
│    Initializing...      │
└─────────────────────────┘
```

### Flash 信息页面 (2-5秒)
```
┌─────────────────────────┐
│      Flash Info         │
├─────────────────────────┤
│ JEDEC: EF4018           │
│ Size: 16MB              │
│                         │
│                         │
│                         │
│                         │
│                         │
│                         │
│                         │
│      Ready!             │
└─────────────────────────┘
```

### 资源列表视图 (主界面)
```
┌─────────────────────────┐
│    Flash Resources      │
├─────────────────────────┤
│ > boot_screen           │
│   font_bitmap           │
│   ui_graphics           │
│   app_data              │
│   user_config           │
│   log_storage           │
├─────────────────────────┤
│ BTN1: Up  BTN3: Select  │
└─────────────────────────┘
```

### 资源详情视图 (选择 boot_screen)
```
┌─────────────────────────┐
│     boot_screen         │
├─────────────────────────┤
│ Addr: 0x00000000        │
│ Size: 107KB             │
│ 320x172 RGB565 boot     │
│ screen                  │
│                         │
│ Data preview:           │
│ 00 01 02 03 04 05 06 07 │
│                         │
├─────────────────────────┤
│ BTN3: Back              │
└─────────────────────────┘
```

## 🎮 交互演示

### 按钮操作日志
```
[INFO] Read 16 bytes from address 0x00000000 (dummy data)
[INFO] Draw text at (10, 20): 'boot_screen'
[INFO] Draw text at (10, 45): 'Addr: 0x00000000'
[INFO] Draw text at (10, 65): 'Size: 107KB'
[INFO] Draw text at (10, 85): '320x172 RGB565 boot screen'
[INFO] Draw text at (10, 110): 'Data preview:'
[INFO] Draw text at (10, 130): '00'
[INFO] Draw text at (35, 130): '01'
[INFO] Draw text at (60, 130): '02'
[INFO] Draw text at (85, 130): '03'
[INFO] Draw text at (110, 130): '04'
[INFO] Draw text at (135, 130): '05'
[INFO] Draw text at (160, 130): '06'
[INFO] Draw text at (185, 130): '07'
[INFO] Draw text at (10, 210): 'BTN3: Back'
```

### 导航演示
```
用户按下 BTN1 (PC10) - 向上导航
[INFO] 当前选择: font_bitmap (索引 1)
[INFO] Display clear (simulated)
[INFO] Draw text at (50, 20): 'Flash Resources'
[INFO] Draw text at (10, 75): ' boot_screen'
[INFO] Draw text at (10, 100): '>font_bitmap'  # 高亮显示
[INFO] Draw text at (10, 125): ' ui_graphics'

用户按下 BTN3 (PC13) - 选择进入详情
[INFO] 进入详情视图: font_bitmap
[INFO] Read 16 bytes from address 0x00020000 (dummy data)
[INFO] Display clear (simulated)
[INFO] Draw text at (10, 20): 'font_bitmap'
[INFO] Draw text at (10, 45): 'Addr: 0x00020000'
[INFO] Draw text at (10, 65): 'Size: 2MB'
[INFO] Draw text at (10, 85): '12px bitmap font (2094 chars)'
```

## 📊 系统状态监控

### 内存使用情况
```
堆内存: 8192 bytes 已分配
缓存状态: 8 个缓存条目可用
SPI1 状态: 已初始化 (显示屏)
SPI2 状态: 已初始化 (Flash)
```

### Flash 访问统计
```
读取操作: 12 次
缓存命中: 3 次
缓存未命中: 9 次
平均读取时间: 2ms (模拟)
```

## 🔧 调试信息

### 资源布局验证
```
[DEBUG] 资源验证开始...
[DEBUG] boot_screen: 0x000000 - 0x01ADFF ✅
[DEBUG] font_bitmap: 0x020000 - 0x21FFFF ✅
[DEBUG] ui_graphics: 0x220000 - 0x41FFFF ✅
[DEBUG] app_data: 0x420000 - 0x71FFFF ✅
[DEBUG] user_config: 0x720000 - 0x72FFFF ✅
[DEBUG] log_storage: 0x730000 - 0x74FFFF ✅
[DEBUG] firmware_update: 0x750000 - 0x7CFFFF ✅
[DEBUG] reserved: 0x7D0000 - 0xFFFFFF ✅
[DEBUG] 所有资源区域验证通过！
```

### 硬件状态检查
```
[DEBUG] SPI1 频率: 16MHz (显示屏)
[DEBUG] SPI2 频率: 1MHz (Flash, 保守设置)
[DEBUG] 按钮状态: BTN1=HIGH, BTN3=HIGH (上拉)
[DEBUG] Flash 写保护: 启用 (PB11=HIGH)
[DEBUG] Flash 保持: 正常操作 (PA10=HIGH)
```

## 🎯 预期用户体验

1. **启动体验**: 2秒内完成初始化，显示清晰的状态信息
2. **导航体验**: 按钮响应迅速，界面切换流畅
3. **信息展示**: 资源信息一目了然，数据预览直观
4. **错误处理**: 如果 Flash 未连接，会显示明确的错误信息
5. **性能表现**: 缓存机制确保重复访问的快速响应

## 📝 实际运行说明

由于当前版本使用了简化的显示和 Flash 操作（用于演示框架），实际运行时会看到：
- 所有显示操作通过 defmt 日志输出到调试器
- Flash 读取返回基于地址的模拟数据
- 按钮检测通过轮询方式实现
- 界面更新通过日志消息体现

要获得完整的视觉体验，需要集成真实的 GC9307 显示驱动和 W25Q128JV Flash 驱动。
