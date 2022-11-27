use crate::program::ui_system::font_manager::{Font, FontWeight};
use crate::program::ui_system::UiConfig;
use imgui::FontConfig;

pub const UI_CONFIG: UiConfig = UiConfig {
    vsync: true,
    hardware_acceleration: Some(true),
};

pub fn base_font_config() -> FontConfig {
    FontConfig {
        //TODO: Configure
        // Oversampling font helps improve text rendering at
        // expense of larger font atlas texture.
        oversample_h: 3,
        oversample_v: 3,
        ..FontConfig::default()
    }
}
