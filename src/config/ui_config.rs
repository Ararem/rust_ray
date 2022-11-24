use crate::program::ui_system::UiConfig;
pub const UI_CONFIG: UiConfig = UiConfig {
    vsync: false,
    hardware_acceleration: Some(true),
};

pub const DEFAULT_FONT_SIZE: f32 = 24f32;