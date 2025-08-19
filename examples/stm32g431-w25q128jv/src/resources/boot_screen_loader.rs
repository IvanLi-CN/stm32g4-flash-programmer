use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use heapless::Vec;
use crate::hardware::flash::FlashManager;

/// Display trait for generic display operations
pub trait DisplayTrait {
    type Error;

    async fn fill_screen(&mut self, color: Rgb565) -> Result<(), Self::Error>;
    async fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: Rgb565) -> Result<(), Self::Error>;

    /// Draw single pixel
    async fn draw_pixel(&mut self, x: u16, y: u16, color: Rgb565) -> Result<(), Self::Error>;
}

/// 开屏图加载器
pub struct BootScreenLoader {
    screen_addr: u32,
    screen_width: u16,
    screen_height: u16,
    screen_size: u32,
    chunk_size: usize,
}

/// 开屏图信息
#[derive(Debug, Clone)]
pub struct BootScreenInfo {
    pub width: u16,
    pub height: u16,
    pub total_size: u32,
    pub pixel_format: PixelFormat,
}

/// 像素格式
#[derive(Debug, Clone)]
pub enum PixelFormat {
    Rgb565,
}

/// 图像块信息
#[derive(Debug)]
pub struct ImageChunk {
    pub chunk_index: usize,
    pub start_x: u16,
    pub start_y: u16,
    pub width: u16,
    pub height: u16,
    pub data_offset: u32,
    pub data_size: usize,
}

impl BootScreenLoader {
    /// 创建新的开屏图加载器
    pub fn new() -> Self {
        Self {
            screen_addr: 0x00000000,    // 开屏图在Flash中的基地址
            screen_width: 320,          // 屏幕宽度
            screen_height: 172,         // 屏幕高度
            screen_size: 320 * 172 * 2, // RGB565格式，每像素2字节
            chunk_size: 2048,           // 每次读取2KB (优化分片大小)
        }
    }

    /// 获取开屏图基本信息
    pub fn get_screen_info(&self) -> BootScreenInfo {
        BootScreenInfo {
            width: self.screen_width,
            height: self.screen_height,
            total_size: self.screen_size,
            pixel_format: PixelFormat::Rgb565,
        }
    }

    /// 计算总共需要多少个块
    pub fn get_total_chunks(&self) -> usize {
        ((self.screen_size as usize) + self.chunk_size - 1) / self.chunk_size
    }

    /// 计算指定块的信息
    pub fn get_chunk_info(&self, chunk_index: usize) -> Result<ImageChunk, &'static str> {
        let total_chunks = self.get_total_chunks();

        if chunk_index >= total_chunks {
            return Err("Chunk index out of range");
        }

        // 计算数据偏移和大小
        let data_offset = (chunk_index * self.chunk_size) as u32;
        let remaining_bytes = self.screen_size - data_offset;
        let data_size = core::cmp::min(self.chunk_size, remaining_bytes as usize);

        // 计算像素数量（RGB565每像素2字节）
        let pixels_in_chunk = data_size / 2;
        let total_pixels_before = (chunk_index * self.chunk_size) / 2;

        // 计算起始位置 - 按行主序排列
        let start_x = (total_pixels_before % (self.screen_width as usize)) as u16;
        let start_y = (total_pixels_before / (self.screen_width as usize)) as u16;

        // 简化处理：每个块都按线性像素序列处理，不强制矩形
        // 这样可以避免复杂的跨行计算错误
        let width = self.screen_width;  // 使用全屏宽度
        let height = 1;                 // 每次处理一个像素序列

        defmt::debug!("Chunk {}: offset=0x{:X}, size={}, pixels={}, start=({},{})",
                     chunk_index, data_offset, data_size, pixels_in_chunk, start_x, start_y);

        Ok(ImageChunk {
            chunk_index,
            start_x,
            start_y,
            width,
            height,
            data_offset,
            data_size,
        })
    }

    /// 读取指定块的数据
    pub async fn read_chunk_data(
        &self,
        chunk_info: &ImageChunk,
        flash_manager: &mut FlashManager
    ) -> Result<heapless::Vec<u8, 2048>, &'static str> {
        let read_addr = self.screen_addr + chunk_info.data_offset;

        defmt::debug!("📖 Reading chunk {} from 0x{:08X}, size: {} bytes",
                     chunk_info.chunk_index, read_addr, chunk_info.data_size);

        let chunk_data = flash_manager.read_data_large(read_addr, chunk_info.data_size).await?;

        if chunk_data.len() < chunk_info.data_size {
            defmt::error!("❌ Failed to read complete chunk: got {} bytes, expected {}",
                         chunk_data.len(), chunk_info.data_size);
            return Err("Incomplete chunk read");
        }

        // 转换为2048字节的Vec
        let mut result = heapless::Vec::new();
        for &byte in &chunk_data {
            result.push(byte).map_err(|_| "Chunk buffer overflow")?;
        }

        defmt::debug!("✅ Chunk {} read successfully: {} bytes",
                     chunk_info.chunk_index, result.len());
        Ok(result)
    }

    /// 将RGB565数据转换为像素颜色数组
    pub fn convert_rgb565_data(
        &self,
        data: &[u8]
    ) -> Result<Vec<Rgb565, 1024>, &'static str> {
        if data.len() % 2 != 0 {
            return Err("RGB565 data length must be even");
        }

        let pixel_count = data.len() / 2;
        if pixel_count > 1024 {
            return Err("Too many pixels for buffer");
        }

        let mut pixels = Vec::new();

        // 学习web工具的RGB565解码方式：data[i] | (data[i+1] << 8)
        for i in (0..data.len()).step_by(2) {
            if i + 1 < data.len() {
                // 按照web工具的方式解码RGB565 (little-endian)
                let rgb565 = data[i] as u16 | ((data[i + 1] as u16) << 8);

                // 提取RGB分量 (与web工具一致的位操作)
                let red = ((rgb565 >> 11) & 0x1F) as u8;    // 5位红色
                let green = ((rgb565 >> 5) & 0x3F) as u8;   // 6位绿色
                let blue = (rgb565 & 0x1F) as u8;           // 5位蓝色

                let color = Rgb565::new(red, green, blue);
                pixels.push(color).map_err(|_| "Pixel buffer full")?;
            }
        }

        defmt::debug!("✅ RGB565 decode: {} bytes -> {} pixels", data.len(), pixels.len());

        Ok(pixels)
    }

    /// 加载并显示完整的开屏图
    pub async fn load_and_display<D>(
        &self,
        display: &mut D,
        flash_manager: &mut FlashManager
    ) -> Result<(), &'static str>
    where
        D: DisplayTrait,
    {
        let total_chunks = self.get_total_chunks();

        defmt::info!("🖼️ Loading boot screen: {}x{} pixels, {} chunks",
                    self.screen_width, self.screen_height, total_chunks);

        // 首先清空屏幕
        defmt::debug!("🧹 Clearing screen...");
        display.fill_screen(Rgb565::BLACK).await.map_err(|_| "Failed to clear screen")?;

        // 分块加载和显示
        for chunk_index in 0..total_chunks {
            // 计算块信息
            let chunk_info = self.get_chunk_info(chunk_index)?;

            // 读取块数据
            let chunk_data = self.read_chunk_data(&chunk_info, flash_manager).await?;

            // 转换为像素数据
            let pixels = self.convert_rgb565_data(&chunk_data)?;

            // 显示块数据
            self.display_chunk(display, &chunk_info, &pixels).await?;

            // 显示详细进度信息
            let progress = ((chunk_index + 1) * 100) / total_chunks;
            let pixels_rendered = (chunk_index + 1) * (self.chunk_size / 2);
            let total_pixels = (self.screen_width as usize) * (self.screen_height as usize);

            defmt::info!("📊 Image render progress: {}% ({}/{} chunks, {}/{} pixels)",
                        progress, chunk_index + 1, total_chunks, pixels_rendered, total_pixels);

            // 减少延迟，提高渲染速度
            embassy_time::Timer::after_millis(1).await;
        }

        defmt::info!("✅ Boot screen loaded successfully!");
        Ok(())
    }

    /// 显示单个图像块 (线性像素序列渲染)
    async fn display_chunk<D>(
        &self,
        display: &mut D,
        chunk_info: &ImageChunk,
        pixels: &[Rgb565]
    ) -> Result<(), &'static str>
    where
        D: DisplayTrait,
    {
        defmt::trace!("🎨 Displaying chunk {} with {} pixels starting from offset 0x{:X}",
                     chunk_info.chunk_index, pixels.len(), chunk_info.data_offset);

        // 计算起始像素位置（基于数据偏移）
        let total_pixels_before = (chunk_info.data_offset / 2) as usize;

        // 按行主序渲染像素
        for (i, &pixel_color) in pixels.iter().enumerate() {
            let absolute_pixel_index = total_pixels_before + i;

            // 计算屏幕坐标（行主序：从左到右，从上到下）
            let pixel_x = (absolute_pixel_index % (self.screen_width as usize)) as u16;
            let pixel_y = (absolute_pixel_index / (self.screen_width as usize)) as u16;

            // 确保坐标在屏幕范围内
            if pixel_x < self.screen_width && pixel_y < self.screen_height {
                display.draw_pixel(pixel_x, pixel_y, pixel_color)
                    .await.map_err(|_| "Failed to draw pixel")?;
            }
        }

        defmt::debug!("✅ Chunk {} rendered: {} pixels from offset 0x{:X}",
                     chunk_info.chunk_index, pixels.len(), chunk_info.data_offset);

        Ok(())
    }

    /// 验证开屏图数据的完整性
    pub async fn verify_screen_data(
        &self,
        flash_manager: &mut FlashManager
    ) -> Result<(), &'static str> {
        defmt::info!("🔍 Verifying boot screen data integrity...");
        defmt::info!("🔍 DEBUG: screen_addr = 0x{:08X}", self.screen_addr);

        // 读取前几个字节检查数据是否存在
        defmt::info!("🔍 DEBUG: About to call read_data_simple");
        let test_data = flash_manager.read_data_simple(self.screen_addr, 16).await?;
        defmt::info!("🔍 DEBUG: read_data_simple completed successfully");

        if test_data.len() < 16 {
            return Err("Failed to read test data");
        }

        // 检查是否全为0xFF（未写入的Flash状态）
        let all_ff = test_data.iter().all(|&b| b == 0xFF);
        if all_ff {
            defmt::warn!("⚠️ Boot screen data appears to be empty (all 0xFF)");
            return Err("Boot screen data not found");
        }

        // 检查是否全为0x00
        let all_zero = test_data.iter().all(|&b| b == 0x00);
        if all_zero {
            defmt::warn!("⚠️ Boot screen data appears to be corrupted (all 0x00)");
            return Err("Boot screen data corrupted");
        }

        defmt::info!("✅ Boot screen data verification passed");
        defmt::debug!("First 16 bytes: {:?}", &test_data[..]);

        Ok(())
    }

    /// 获取开屏图的统计信息
    pub async fn get_screen_stats(
        &self,
        flash_manager: &mut FlashManager
    ) -> Result<ScreenStats, &'static str> {
        // 采样一些像素来分析图像
        let sample_size = 256; // 采样256个像素
        let sample_data = flash_manager.read_data_simple(self.screen_addr, sample_size * 2).await?;

        let pixels = self.convert_rgb565_data(&sample_data)?;

        let mut red_sum = 0u32;
        let mut green_sum = 0u32;
        let mut blue_sum = 0u32;

        for pixel in &pixels {
            red_sum += pixel.r() as u32;
            green_sum += pixel.g() as u32;
            blue_sum += pixel.b() as u32;
        }

        let pixel_count = pixels.len() as u32;

        Ok(ScreenStats {
            width: self.screen_width,
            height: self.screen_height,
            total_size: self.screen_size,
            sampled_pixels: pixel_count,
            avg_red: red_sum / pixel_count,
            avg_green: green_sum / pixel_count,
            avg_blue: blue_sum / pixel_count,
        })
    }
}

/// 开屏图统计信息
#[derive(Debug)]
pub struct ScreenStats {
    pub width: u16,
    pub height: u16,
    pub total_size: u32,
    pub sampled_pixels: u32,
    pub avg_red: u32,
    pub avg_green: u32,
    pub avg_blue: u32,
}
