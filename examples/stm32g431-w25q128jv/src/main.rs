#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    spi::{self, Spi},
    gpio::{Level, Output, Speed},
    time::Hertz,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::Mutex,
};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::RgbColor,
};
use panic_probe as _;
use defmt_rtt as _;
use static_cell::StaticCell;

mod hardware;
mod resources;
mod ui;

use hardware::{flash::FlashManager, display::DisplayManager};
// Resource layout removed - no fonts in firmware

// Static allocations
static SPI1_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>>> = StaticCell::new();
static SPI2_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<'static, embassy_stm32::mode::Async>>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    // Add interrupt bindings as needed
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("STM32G431CBU6 Flash Content Viewer starting...");

    // Prevent optimization from removing all code
    cortex_m::asm::nop();

    // Initialize STM32
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 170 MHz system clock
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
    }
    let p = embassy_stm32::init(config);
    defmt::info!("STM32G431CBU6 initialized successfully");

    // Initialize SPI1 for display (GC9307)
    let mut spi1_config = spi::Config::default();
    spi1_config.frequency = Hertz(16_000_000); // 16MHz
    let spi1 = Spi::new_txonly(
        p.SPI1,
        p.PB3,  // SCK
        p.PB5,  // MOSI
        p.DMA1_CH3, // TX DMA
        spi1_config,
    );
    let spi1_bus = SPI1_BUS.init(Mutex::new(spi1));

    // Initialize SPI2 for Flash (W25Q128JV)
    let mut spi2_config = spi::Config::default();
    spi2_config.frequency = Hertz(500_000); // 500kHz for Flash (very conservative)

    // W25Q128JV requires SPI Mode 0 (CPOL=0, CPHA=0) - this is the default
    // spi2_config.mode is already Mode 0 by default
    let spi2 = Spi::new(
        p.SPI2,
        p.PB13, // SCK
        p.PB15, // MOSI
        p.PB14, // MISO
        p.DMA2_CH3, // TX DMA
        p.DMA2_CH2, // RX DMA
        spi2_config,
    );
    let spi2_bus = SPI2_BUS.init(Mutex::new(spi2));

    // Display control pins
    let display_cs = Output::new(p.PA15, Level::High, Speed::High);
    let display_dc = Output::new(p.PC14, Level::Low, Speed::High);
    let display_rst = Output::new(p.PC15, Level::Low, Speed::High);

    // Flash control pins
    let flash_cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);
    let _flash_wp = Output::new(p.PB11, Level::High, Speed::VeryHigh); // Write protect (HIGH = enabled)
    let _flash_hold = Output::new(p.PA10, Level::High, Speed::VeryHigh); // Hold (HIGH = normal operation)

    defmt::info!("Hardware pins configured");

    // Initialize display
    let mut display_manager = DisplayManager::new();
    match display_manager.initialize(spi1_bus, display_cs, display_dc, display_rst).await {
        Ok(()) => {
            defmt::info!("✅ Display initialized successfully");

            // Step 1: Render checkerboard pattern first
            defmt::info!("🎨 Step 1: Rendering checkerboard pattern...");
            match display_manager.draw_checkerboard().await {
                Ok(()) => defmt::info!("✅ Checkerboard pattern rendered successfully"),
                Err(e) => defmt::error!("❌ Failed to render checkerboard: {}", e),
            }

            // Wait to show checkerboard
            Timer::after(Duration::from_millis(2000)).await;

            defmt::info!("🚀 Display driver ready for font and image rendering!");
        }
        Err(e) => {
            defmt::error!("❌ Display initialization failed: {}", e);
        }
    }

    // Wait a moment for startup screen
    Timer::after(Duration::from_millis(2000)).await;

    // Initialize Flash
    let mut flash_manager = FlashManager::new();
    match flash_manager.initialize(spi2_bus, flash_cs).await {
        Ok(()) => {
            defmt::info!("✅ Flash initialized successfully");

            // First, test SPI communication by reading Flash chip ID
            defmt::info!("=== Flash SPI Communication Test ===");
            defmt::info!("🔍 Testing Flash chip ID (JEDEC ID)...");

            match flash_manager.read_jedec_id().await {
                Ok(id) => {
                    defmt::info!("✅ Flash JEDEC ID: {:02X} {:02X} {:02X} (Manufacturer: 0x{:02X}, Device: 0x{:04X})",
                          id[0], id[1], id[2], id[0], (id[1] as u16) << 8 | id[2] as u16);

                    // Expected for W25Q128JV: [0xEF, 0x40, 0x18]
                    if id == [0xEF, 0x40, 0x18] {
                        defmt::info!("✅ Confirmed: W25Q128JV Flash chip detected!");
                    } else {
                        defmt::warn!("⚠️  Unexpected Flash chip ID. Expected W25Q128JV [0xEF, 0x40, 0x18]");
                    }
                },
                Err(e) => {
                    defmt::error!("❌ Failed to read Flash JEDEC ID: {}", e);
                    defmt::error!("❌ SPI communication may be broken!");
                }
            }

            // Initialize 16px font renderer
            defmt::info!("=== Initializing 16px Font System ===");
            match display_manager.initialize_16px_font(&mut flash_manager).await {
                Ok(()) => {
                    defmt::info!("✅ 16px font system initialized successfully");
                },
                Err(e) => {
                    defmt::error!("❌ Failed to initialize 16px font: {}", e);
                }
            }

            // Skip boot screen for now - will show at the end

            // First, let's explore what's actually in Flash
            defmt::info!("=== Flash Content Exploration ===");

            // Check Flash at address 0x0
            let data_0 = flash_manager.read_data_simple(0x0, 32).await.unwrap_or_default();
            defmt::info!("Flash at 0x00000000: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                data_0.get(0).unwrap_or(&0), data_0.get(1).unwrap_or(&0), data_0.get(2).unwrap_or(&0), data_0.get(3).unwrap_or(&0),
                data_0.get(4).unwrap_or(&0), data_0.get(5).unwrap_or(&0), data_0.get(6).unwrap_or(&0), data_0.get(7).unwrap_or(&0));

            // Check Flash at address 0x1000 (4KB)
            let data_1k = flash_manager.read_data_simple(0x1000, 32).await.unwrap_or_default();
            defmt::info!("Flash at 0x00001000: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                data_1k.get(0).unwrap_or(&0), data_1k.get(1).unwrap_or(&0), data_1k.get(2).unwrap_or(&0), data_1k.get(3).unwrap_or(&0),
                data_1k.get(4).unwrap_or(&0), data_1k.get(5).unwrap_or(&0), data_1k.get(6).unwrap_or(&0), data_1k.get(7).unwrap_or(&0));

            // Check Flash at address 0x10000 (64KB)
            let data_64k = flash_manager.read_data_simple(0x10000, 32).await.unwrap_or_default();
            defmt::info!("Flash at 0x00010000: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                data_64k.get(0).unwrap_or(&0), data_64k.get(1).unwrap_or(&0), data_64k.get(2).unwrap_or(&0), data_64k.get(3).unwrap_or(&0),
                data_64k.get(4).unwrap_or(&0), data_64k.get(5).unwrap_or(&0), data_64k.get(6).unwrap_or(&0), data_64k.get(7).unwrap_or(&0));

            // Check Flash at address 0x20000 (128KB) - where we expect font data
            let data_128k = flash_manager.read_data_simple(0x20000, 32).await.unwrap_or_default();
            defmt::info!("Flash at 0x00020000: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                data_128k.get(0).unwrap_or(&0), data_128k.get(1).unwrap_or(&0), data_128k.get(2).unwrap_or(&0), data_128k.get(3).unwrap_or(&0),
                data_128k.get(4).unwrap_or(&0), data_128k.get(5).unwrap_or(&0), data_128k.get(6).unwrap_or(&0), data_128k.get(7).unwrap_or(&0));

            // Check if Flash is all 0xFF (erased)
            let data_ff = flash_manager.read_data_simple(0x100000, 32).await.unwrap_or_default();
            defmt::info!("Flash at 0x00100000: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
                data_ff.get(0).unwrap_or(&0), data_ff.get(1).unwrap_or(&0), data_ff.get(2).unwrap_or(&0), data_ff.get(3).unwrap_or(&0),
                data_ff.get(4).unwrap_or(&0), data_ff.get(5).unwrap_or(&0), data_ff.get(6).unwrap_or(&0), data_ff.get(7).unwrap_or(&0));

            // Skip font data generation - no fonts stored in firmware
            defmt::info!("Skipping font data generation (no fonts in firmware)");

            // Show new 16px font interface
            // Always show the interface since Flash is initialized
            display_manager.clear(Rgb565::BLACK).await.unwrap_or_default();

            defmt::info!("=== Testing 16px Font Rendering ===");

            // Title with 16px font
            match display_manager.draw_text_16px("Flash Viewer 16px", 10, 20, Rgb565::WHITE, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Title rendered with 16px font"),
                Err(e) => defmt::error!("❌ Failed to render title: {}", e),
            }

            // Flash info with 16px font
            match display_manager.draw_text_16px("JEDEC: EF4018", 10, 45, Rgb565::CYAN, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Flash info rendered"),
                Err(e) => defmt::error!("❌ Failed to render flash info: {}", e),
            }

            match display_manager.draw_text_16px("Size: 16MB", 10, 70, Rgb565::GREEN, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Size info rendered"),
                Err(e) => defmt::error!("❌ Failed to render size info: {}", e),
            }

            // Chinese character test with 16px font
            match display_manager.draw_text_16px("中文显示测试", 10, 95, Rgb565::MAGENTA, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Chinese text rendered with 16px font"),
                Err(e) => defmt::error!("❌ Failed to render Chinese text: {}", e),
            }

            // Mixed text test
            match display_manager.draw_text_16px("Hello 世界!", 10, 120, Rgb565::YELLOW, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Mixed text rendered"),
                Err(e) => defmt::error!("❌ Failed to render mixed text: {}", e),
            }

            // Status
            match display_manager.draw_text_16px("16px Ready!", 10, 145, Rgb565::WHITE, &mut flash_manager).await {
                Ok(()) => defmt::info!("✅ Status rendered"),
                Err(e) => defmt::error!("❌ Failed to render status: {}", e),
            }

            defmt::info!("✅ Step 2: Text rendering completed");
        }
        Err(e) => {
            defmt::error!("❌ Flash initialization failed: {}", e);
            // Can't show error with text since Flash is needed for font data
            // Just show a red screen as error indicator
            display_manager.clear(Rgb565::RED).await.unwrap_or_default();
            // Draw some simple rectangles to indicate error
            display_manager.fill_rect(50, 50, 220, 80, Rgb565::WHITE).await.unwrap_or_default();
            display_manager.fill_rect(60, 60, 200, 60, Rgb565::RED).await.unwrap_or_default();
        }
    }

    // Wait before final step
    Timer::after(Duration::from_millis(3000)).await;

    // Step 3: Finally render the boot screen image from Flash and keep it displayed
    defmt::info!("🖼️ Step 3: Rendering boot screen image from Flash...");

    // 🎬 Start three-screen cycling display system
    defmt::info!("🎬 Starting three-screen cycling display system...");

    let mut screen_index = 0u8;
    let screen_duration = Duration::from_millis(4000); // 4 seconds per screen

    loop {
        match screen_index {
            // 第一屏：启动图片屏幕
            0 => {
                defmt::info!("📺 Screen 1/3: Boot Screen Image");
                match display_manager.show_boot_screen(&mut flash_manager).await {
                    Ok(()) => {
                        defmt::info!("✅ Boot screen image displayed successfully");
                    }
                    Err(e) => {
                        defmt::error!("❌ Failed to show boot screen image: {}", e);
                        display_manager.clear(Rgb565::RED).await.unwrap_or_default();
                        display_manager.draw_text_16px("Boot Image Failed", 10, 100, Rgb565::WHITE, &mut flash_manager).await.unwrap_or_default();
                    }
                }
            }

            // 第二屏：12px字体文字屏幕
            1 => {
                defmt::info!("📺 Screen 2/3: 12px Font Text");
                display_manager.clear(Rgb565::BLACK).await.unwrap_or_default();

                // 显示系统信息
                display_manager.draw_text("STM32G431 Flash Viewer", 10, 20, Rgb565::WHITE, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Flash: W25Q128JV (16MB)", 10, 40, Rgb565::CYAN, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Display: 320x172 RGB565", 10, 60, Rgb565::GREEN, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Status: Running OK", 10, 80, Rgb565::YELLOW, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Memory: Boot+Font+Data", 10, 100, Rgb565::MAGENTA, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Mode: Cycling Display", 10, 120, Rgb565::WHITE, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text("Screen 2/3 - 12px Font", 10, 150, Rgb565::BLUE, &mut flash_manager).await.unwrap_or_default();
            }

            // 第三屏：16px字体文字屏幕
            2 => {
                defmt::info!("📺 Screen 3/3: 16px Font Text");
                display_manager.clear(Rgb565::BLACK).await.unwrap_or_default();

                // 显示版本和状态信息
                display_manager.draw_text_16px("Flash Content Viewer", 10, 20, Rgb565::WHITE, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text_16px("Version: v1.0.0", 10, 45, Rgb565::CYAN, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text_16px("Build: 2024-08-18", 10, 70, Rgb565::GREEN, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text_16px("MCU: STM32G431CBU6", 10, 95, Rgb565::YELLOW, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text_16px("Freq: 170MHz", 10, 120, Rgb565::MAGENTA, &mut flash_manager).await.unwrap_or_default();
                display_manager.draw_text_16px("Screen 3/3 - 16px", 10, 145, Rgb565::BLUE, &mut flash_manager).await.unwrap_or_default();
            }

            _ => {
                // 重置到第一屏
                screen_index = 0;
                continue;
            }
        }

        // 等待指定时间后切换到下一屏
        Timer::after(screen_duration).await;

        // 切换到下一屏
        screen_index = (screen_index + 1) % 3;

        defmt::debug!("🔄 Switching to next screen...");
    }
}

// Font-related functions removed - no fonts stored in firmware

// All font-related display functions removed

// All text formatting functions removed - no fonts in firmware

// End of file - all font-related code removed
