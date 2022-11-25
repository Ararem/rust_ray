use crate::program::ui_system::UiConfig;
use imgui::FontConfig;

pub const UI_CONFIG: UiConfig = UiConfig {
    vsync: false,
    hardware_acceleration: Some(true),
};

pub const RENDERED_FONT_SIZE:f32 = 64f32;
pub const DEFAULT_FONT_SIZE: f32 = 24f32;

// Fixed font size. Note imgui_winit_support uses "logical
// pixels", which are physical pixels scaled by the devices
// scaling factor. Meaning, 15.0 pixels should look the same size
// on two different screens, and thus we do not need to scale this
// value (as the scaling is handled by winit)
pub fn base_font_config() -> FontConfig {
    FontConfig {
        //TODO: Configure
        // Oversampling font helps improve text rendering at
        // expense of larger font atlas texture.
        oversample_h: 4,
        oversample_v: 4,
        ..FontConfig::default()
    }
}
