# STM32 Custom Font Usage Guide

This guide shows how to use the custom fonts in your STM32G431CBU6 project.

## Quick Start

### 1. Include the Font Renderers

```rust
use crate::font_renderer_digit::FontRendererDigit;
use crate::font_renderer_ascii::FontRendererASCII;
use crate::flash_manager::FlashManager;
```

### 2. Initialize Font Renderers

```rust
// Create font renderers
let mut digit_font = FontRendererDigit::new();
let mut ascii_font = FontRendererASCII::new();

// Initialize with flash manager
digit_font.initialize(&mut flash_manager).await?;
ascii_font.initialize(&mut flash_manager).await?;
```

### 3. Render Numbers (24×48 pixels)

```rust
// Prepare display buffer
let mut display_buffer = [0u8; 320 * 240]; // 320x240 display

// Render voltage reading
let voltage = "20.5"; // Voltage value as string
digit_font.render_number_string(
    &mut flash_manager,
    voltage,
    &mut display_buffer,
    320, // buffer width
    240, // buffer height
    50,  // x position
    100, // y position
).await?;
```

### 4. Render Text (16×24 pixels)

```rust
// Render menu label
ascii_font.render_text_string(
    &mut flash_manager,
    "Voltage:",
    &mut display_buffer,
    320, 240,
    10,  // x position
    100, // y position
).await?;
```

## Complete Example: Power Display

```rust
use crate::font_renderer_digit::FontRendererDigit;
use crate::font_renderer_ascii::FontRendererASCII;
use crate::flash_manager::FlashManager;

pub struct PowerDisplay {
    digit_font: FontRendererDigit,
    ascii_font: FontRendererASCII,
    display_buffer: [u8; 320 * 240],
}

impl PowerDisplay {
    pub fn new() -> Self {
        Self {
            digit_font: FontRendererDigit::new(),
            ascii_font: FontRendererASCII::new(),
            display_buffer: [0; 320 * 240],
        }
    }

    pub async fn initialize(&mut self, flash_manager: &mut FlashManager) -> Result<(), &'static str> {
        self.digit_font.initialize(flash_manager).await?;
        self.ascii_font.initialize(flash_manager).await?;
        Ok(())
    }

    pub async fn update_display(
        &mut self,
        flash_manager: &mut FlashManager,
        voltage: f32,
        current: f32,
        power: f32,
    ) -> Result<(), &'static str> {
        // Clear display buffer
        self.display_buffer.fill(0);

        // Format values as strings
        let voltage_str = heapless::String::<16>::from(voltage);
        let current_str = heapless::String::<16>::from(current);
        let power_str = heapless::String::<16>::from(power);

        // Render voltage
        self.ascii_font.render_text_string(
            flash_manager, "Voltage:", &mut self.display_buffer,
            320, 240, 10, 20
        ).await?;
        
        self.digit_font.render_number_string(
            flash_manager, &voltage_str, &mut self.display_buffer,
            320, 240, 120, 10
        ).await?;
        
        self.ascii_font.render_text_string(
            flash_manager, "V", &mut self.display_buffer,
            320, 240, 250, 20
        ).await?;

        // Render current
        self.ascii_font.render_text_string(
            flash_manager, "Current:", &mut self.display_buffer,
            320, 240, 10, 80
        ).await?;
        
        self.digit_font.render_number_string(
            flash_manager, &current_str, &mut self.display_buffer,
            320, 240, 120, 70
        ).await?;
        
        self.ascii_font.render_text_string(
            flash_manager, "A", &mut self.display_buffer,
            320, 240, 250, 80
        ).await?;

        // Render power
        self.ascii_font.render_text_string(
            flash_manager, "Power:", &mut self.display_buffer,
            320, 240, 10, 140
        ).await?;
        
        self.digit_font.render_number_string(
            flash_manager, &power_str, &mut self.display_buffer,
            320, 240, 120, 130
        ).await?;
        
        self.ascii_font.render_text_string(
            flash_manager, "W", &mut self.display_buffer,
            320, 240, 250, 140
        ).await?;

        Ok(())
    }

    pub fn get_display_buffer(&self) -> &[u8] {
        &self.display_buffer
    }
}
```

## Font Specifications

### Digital Font (24×48 pixels)
- **Characters**: `0123456789-.`
- **Use cases**: Voltage, current, power readings
- **Flash address**: `0x7D0000`
- **Character width**: 24 pixels (fixed)
- **Character height**: 48 pixels (fixed)

### ASCII Font (16×24 pixels)
- **Characters**: ASCII 32-126 (space to tilde)
- **Use cases**: Menu labels, status text, units
- **Flash address**: `0x7D1000`
- **Character width**: 16 pixels (fixed)
- **Character height**: 24 pixels (fixed)

## Performance Tips

### 1. Character Caching
- Digital font: All 12 characters are pre-cached
- ASCII font: 32 most recently used characters are cached
- Cache misses trigger Flash reads with binary search

### 2. Buffer Management
```rust
// Use appropriate buffer sizes
let mut small_buffer = [0u8; 128 * 64];  // For small displays
let mut large_buffer = [0u8; 320 * 240]; // For larger displays

// Clear only when necessary
buffer.fill(0); // Clear entire buffer
```

### 3. Batch Rendering
```rust
// Render related text together to minimize Flash access
ascii_font.render_text_string(flash_manager, "Voltage:", buffer, 320, 240, 10, 20).await?;
digit_font.render_number_string(flash_manager, "20.5", buffer, 320, 240, 120, 10).await?;
ascii_font.render_text_string(flash_manager, "V", buffer, 320, 240, 250, 20).await?;
```

## Error Handling

```rust
match digit_font.render_number_string(flash_manager, "12.34", buffer, 320, 240, 10, 10).await {
    Ok(width) => {
        defmt::info!("Rendered with width: {} pixels", width);
    }
    Err(e) => {
        defmt::error!("Font rendering failed: {}", e);
        // Handle error (fallback display, retry, etc.)
    }
}
```

## Testing

Use the included font test module:

```rust
use crate::font_test::FontTest;

let mut font_test = FontTest::new();
font_test.run_all_tests(&mut flash_manager).await?;
```

## Memory Usage

- **Digital font**: ~1.8KB in Flash
- **ASCII font**: ~5.5KB in Flash
- **Runtime cache**: ~640 bytes RAM (digit) + ~1KB RAM (ASCII)
- **Display buffer**: Depends on your display size

## Integration with Display Driver

```rust
// After rendering to buffer, send to display
let display_data = power_display.get_display_buffer();
display_driver.update_screen(display_data).await?;
```

---
*Generated for STM32G431CBU6 PD-Sink Project*
