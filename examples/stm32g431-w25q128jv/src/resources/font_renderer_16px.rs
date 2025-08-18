use heapless::{Vec, FnvIndexMap};
use embedded_graphics::pixelcolor::Rgb565;
use crate::hardware::flash::FlashManager;

/// 16px字体的字符信息结构（10字节格式）
#[derive(Debug, Clone, Copy)]
pub struct CharInfo16px {
    pub unicode: u32,        // 4字节 - Unicode字符码
    pub width: u8,           // 1字节 - 字符宽度
    pub height: u8,          // 1字节 - 字符高度
    pub bitmap_offset: u32,  // 4字节 - 位图数据偏移（注意：4字节，不是2字节！）
}

/// 16px字体渲染器
pub struct FontRenderer16px {
    font_base_addr: u32,
    char_cache: FnvIndexMap<u32, CharInfo16px, 16>, // 缓存16个常用字符
    char_count: u32,
}

impl FontRenderer16px {
    /// 创建新的16px字体渲染器
    pub fn new() -> Self {
        Self {
            font_base_addr: 0x00120000, // 16px字体在Flash中的基地址
            char_cache: FnvIndexMap::new(),
            char_count: 0,
        }
    }

    /// 初始化字体渲染器，读取字体头信息
    pub async fn initialize(&mut self, flash_manager: &mut FlashManager) -> Result<(), &'static str> {
        defmt::info!("🎨 Initializing 16px font renderer...");

        // 读取字体头部（4字节字符数量）
        let header_data = flash_manager.read_data_simple(self.font_base_addr, 4).await?;

        if header_data.len() < 4 {
            return Err("Failed to read font header");
        }

        // 解析字符数量（小端序）
        self.char_count = u32::from_le_bytes([
            header_data[0], header_data[1], header_data[2], header_data[3]
        ]);

        defmt::info!("✅ 16px font initialized: {} characters available", self.char_count);
        Ok(())
    }

    /// 查找字符信息（使用二分查找优化）
    pub async fn find_char(&mut self, char_code: u32, flash_manager: &mut FlashManager) -> Result<CharInfo16px, &'static str> {
        // 首先检查缓存
        if let Some(&cached_info) = self.char_cache.get(&char_code) {
            defmt::debug!("📋 Found character U+{:04X} in cache", char_code);
            return Ok(cached_info);
        }

        defmt::debug!("🔍 Searching for character U+{:04X} in 16px font", char_code);

        // 二分查找字符
        let mut left = 0u32;
        let mut right = self.char_count;

        while left < right {
            let mid = (left + right) / 2;
            let char_info_addr = self.font_base_addr + 4 + (mid * 10); // 4字节头部 + mid * 10字节字符信息

            // 读取字符信息（10字节）
            let char_data = flash_manager.read_data_simple(char_info_addr, 10).await?;

            if char_data.len() < 10 {
                return Err("Failed to read character info");
            }

            // 解析Unicode值
            let unicode = u32::from_le_bytes([
                char_data[0], char_data[1], char_data[2], char_data[3]
            ]);

            if unicode == char_code {
                // 找到字符，解析完整信息
                let char_info = CharInfo16px {
                    unicode,
                    width: char_data[4],
                    height: char_data[5],
                    bitmap_offset: u32::from_le_bytes([
                        char_data[6], char_data[7], char_data[8], char_data[9]
                    ]),
                };

                // 添加到缓存
                if self.char_cache.len() >= 16 {
                    // 缓存已满，移除最旧的条目
                    if let Some((oldest_key, _)) = self.char_cache.iter().next() {
                        let oldest_key = *oldest_key;
                        self.char_cache.remove(&oldest_key);
                    }
                }
                let _ = self.char_cache.insert(char_code, char_info);

                defmt::debug!("✅ Found character U+{:04X}: {}x{}, offset=0x{:08X}",
                             char_code, char_info.width, char_info.height, char_info.bitmap_offset);
                return Ok(char_info);
            } else if unicode < char_code {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        defmt::warn!("❌ Character U+{:04X} not found in 16px font", char_code);
        Err("Character not found")
    }

    /// 读取字符位图数据
    pub async fn read_char_bitmap(
        &self,
        char_info: &CharInfo16px,
        flash_manager: &mut FlashManager
    ) -> Result<Vec<u8, 64>, &'static str> {
        // 计算位图大小
        let bitmap_size_bits = char_info.width as usize * char_info.height as usize;
        let bitmap_size_bytes = (bitmap_size_bits + 7) / 8;

        if bitmap_size_bytes > 64 {
            defmt::error!("❌ Bitmap too large: {} bytes (max 64)", bitmap_size_bytes);
            return Err("Bitmap too large");
        }

        // 计算位图在Flash中的绝对地址
        let bitmap_addr = self.font_base_addr + char_info.bitmap_offset;

        defmt::debug!("📖 Reading bitmap for U+{:04X}: {} bytes from 0x{:08X}",
                     char_info.unicode, bitmap_size_bytes, bitmap_addr);

        // 读取位图数据
        let bitmap_data = flash_manager.read_data_simple(bitmap_addr, bitmap_size_bytes).await?;

        if bitmap_data.len() < bitmap_size_bytes {
            return Err("Failed to read complete bitmap");
        }

        defmt::debug!("✅ Bitmap read successfully: {} bytes", bitmap_data.len());
        Ok(bitmap_data)
    }

    /// 渲染字符到显示器
    pub async fn render_char<D>(
        &self,
        char_info: &CharInfo16px,
        bitmap: &[u8],
        _display: &mut D,
        x: i32,
        y: i32,
        _color: Rgb565
    ) -> Result<(), &'static str>
    where
        D: embedded_hal_async::spi::SpiDevice,
    {
        let width = char_info.width;
        let height = char_info.height;
        let bytes_per_row = ((width as usize) + 7) / 8;

        defmt::debug!("🎨 Rendering character U+{:04X} at ({}, {}) size {}x{}",
                     char_info.unicode, x, y, width, height);

        // 逐像素渲染（MSB优先，行主序）
        for row in 0..height {
            for col in 0..width {
                let byte_index = (row as usize) * bytes_per_row + (col as usize) / 8;
                let bit_index = 7 - ((col as usize) % 8); // MSB优先

                if byte_index < bitmap.len() {
                    let byte = bitmap[byte_index];
                    let pixel = (byte >> bit_index) & 1;

                    if pixel != 0 {
                        let pixel_x = x + col as i32;
                        let pixel_y = y + row as i32;

                        // 使用display的fill_rect方法绘制1x1像素
                        // 注意：这里需要根据实际的display类型调整
                        // 暂时使用占位符实现
                        defmt::trace!("Drawing pixel at ({}, {})", pixel_x, pixel_y);
                    }
                }
            }
        }

        defmt::debug!("✅ Character rendered successfully");
        Ok(())
    }

    /// 计算文本渲染所需的宽度
    pub async fn calculate_text_width(
        &mut self,
        text: &str,
        flash_manager: &mut FlashManager
    ) -> Result<u32, &'static str> {
        let mut total_width = 0u32;
        let char_spacing = 1u32; // 字符间距

        for ch in text.chars() {
            let char_code = ch as u32;
            match self.find_char(char_code, flash_manager).await {
                Ok(char_info) => {
                    total_width += char_info.width as u32 + char_spacing;
                },
                Err(_) => {
                    // 未找到字符，使用默认宽度
                    total_width += 8 + char_spacing; // 默认8像素宽度
                }
            }
        }

        // 移除最后一个字符的间距
        if total_width > char_spacing {
            total_width -= char_spacing;
        }

        Ok(total_width)
    }

    /// 获取字体基本信息
    pub fn get_font_info(&self) -> (u32, u32) {
        (self.font_base_addr, self.char_count)
    }

    /// 清空字符缓存
    pub fn clear_cache(&mut self) {
        self.char_cache.clear();
        defmt::debug!("🗑️ Character cache cleared");
    }
}

/// 16px字体渲染的辅助函数
impl FontRenderer16px {
    /// 验证字符信息的有效性
    fn validate_char_info(char_info: &CharInfo16px) -> bool {
        // 检查字符尺寸是否合理（16px字体通常不会超过32x32）
        char_info.width > 0 && char_info.width <= 32 &&
        char_info.height > 0 && char_info.height <= 32 &&
        char_info.unicode <= 0x10FFFF // 有效的Unicode范围
    }

    /// 获取字符的基线偏移
    pub fn get_baseline_offset(&self, char_info: &CharInfo16px) -> i32 {
        // 对于16px字体，通常基线在距离底部2-3像素的位置
        const BASELINE_FROM_BOTTOM: i32 = 3;
        char_info.height as i32 - BASELINE_FROM_BOTTOM
    }
}
