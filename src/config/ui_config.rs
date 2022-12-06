use imgui::FontConfig;
use imgui_winit_support::winit::dpi::{LogicalSize, Size};

// pub const UI_CONFIG: UiConfig = UiConfig {
//     vsync: false,
//     hardware_acceleration: Some(true),
//     default_window_size: Size::Logical(LogicalSize { width: 1600.0, height: 900.0 })
// };

pub const DEFAULT_FONT_SIZE: f32 = 20f32;
pub const MIN_FONT_SIZE: f32 = 8f32;
pub const MAX_FONT_SIZE: f32 = 128f32;

pub fn base_font_config() -> FontConfig {
    FontConfig {
        // Oversampling font should help improve text rendering at expense of larger font atlas texture.
        // Personally, I can't tell the difference
        oversample_h: 1,
        oversample_v: 1,
        ..FontConfig::default()
    }
}

/// Colour arrays for use with [`imgui`]
pub mod colours {
    pub const COLOUR_GOOD: [f32; 4] = [1.0, 0.82, 0.0, 1.0];
    pub const COLOUR_WARNING: [f32; 4] = [1.0, 0.82, 0.0, 1.0];
    pub const COLOUR_ERROR: [f32; 4] = [1.0, 0.47, 0.0, 1.0];
    pub const COLOUR_SEVERE_ERROR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
}