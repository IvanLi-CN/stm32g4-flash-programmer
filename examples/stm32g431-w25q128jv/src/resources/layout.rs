/// W25Q128JV Flash memory layout constants
/// Based on the resource layout from assets/memory_map.txt

/// Boot screen resource (RGB565 format, 320x172 pixels)
pub const BOOT_SCREEN_ADDR: u32 = 0x000000;
pub const BOOT_SCREEN_SIZE: u32 = 110_080; // 320 * 172 * 2 bytes

/// Font bitmap resource (12px bitmap font, 2094 characters)
pub const FONT_BITMAP_ADDR: u32 = 0x00020000;
pub const FONT_BITMAP_SIZE: u32 = 2_097_152; // 2MB allocated space

/// UI graphics resource
pub const UI_GRAPHICS_ADDR: u32 = 0x00220000;
pub const UI_GRAPHICS_SIZE: u32 = 2_097_152; // 2MB

/// Application data storage
pub const APP_DATA_ADDR: u32 = 0x00420000;
pub const APP_DATA_SIZE: u32 = 3_145_728; // 3MB

/// User configuration storage
pub const USER_CONFIG_ADDR: u32 = 0x00720000;
pub const USER_CONFIG_SIZE: u32 = 65_536; // 64KB

/// Log storage
pub const LOG_STORAGE_ADDR: u32 = 0x00730000;
pub const LOG_STORAGE_SIZE: u32 = 131_072; // 128KB

/// Firmware update storage
pub const FIRMWARE_UPDATE_ADDR: u32 = 0x00750000;
pub const FIRMWARE_UPDATE_SIZE: u32 = 524_288; // 512KB

/// Reserved area
pub const RESERVED_ADDR: u32 = 0x007D0000;
pub const RESERVED_SIZE: u32 = 8_585_216; // 8.2MB

/// Resource information structure
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub name: &'static str,
    pub address: u32,
    pub size: u32,
    pub description: &'static str,
}

/// All available resources
pub const RESOURCES: &[ResourceInfo] = &[
    ResourceInfo {
        name: "boot_screen",
        address: BOOT_SCREEN_ADDR,
        size: BOOT_SCREEN_SIZE,
        description: "320x172 RGB565 boot screen",
    },
    ResourceInfo {
        name: "font_bitmap",
        address: FONT_BITMAP_ADDR,
        size: FONT_BITMAP_SIZE,
        description: "12px bitmap font (2094 chars)",
    },
    ResourceInfo {
        name: "ui_graphics",
        address: UI_GRAPHICS_ADDR,
        size: UI_GRAPHICS_SIZE,
        description: "UI graphics and icons",
    },
    ResourceInfo {
        name: "app_data",
        address: APP_DATA_ADDR,
        size: APP_DATA_SIZE,
        description: "Application data storage",
    },
    ResourceInfo {
        name: "user_config",
        address: USER_CONFIG_ADDR,
        size: USER_CONFIG_SIZE,
        description: "User configuration",
    },
    ResourceInfo {
        name: "log_storage",
        address: LOG_STORAGE_ADDR,
        size: LOG_STORAGE_SIZE,
        description: "System and error logs",
    },
    ResourceInfo {
        name: "firmware_update",
        address: FIRMWARE_UPDATE_ADDR,
        size: FIRMWARE_UPDATE_SIZE,
        description: "Firmware update storage",
    },
    ResourceInfo {
        name: "reserved",
        address: RESERVED_ADDR,
        size: RESERVED_SIZE,
        description: "Reserved area",
    },
];

/// Get resource by name
pub fn get_resource_by_name(name: &str) -> Option<&'static ResourceInfo> {
    RESOURCES.iter().find(|r| r.name == name)
}

/// Get resource by address
pub fn get_resource_by_address(address: u32) -> Option<&'static ResourceInfo> {
    RESOURCES.iter().find(|r| {
        address >= r.address && address < (r.address + r.size)
    })
}
