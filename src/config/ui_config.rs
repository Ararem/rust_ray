use crate::program::ui_system::font_manager::{Font, FontWeight};
use crate::program::ui_system::UiConfig;
use imgui::FontConfig;

pub const UI_CONFIG: UiConfig = UiConfig {
    vsync: true,
    hardware_acceleration: Some(true),
};

pub const FONT_SIZES: [f32; 8] = [10f32, 12f32, 16f32, 24f32, 32f32, 40f32, 48f32, 64f32];
pub const DEFAULT_FONT_SIZE_INDEX: usize = 3;

pub const BUILTIN_FONTS: &[Font] = &[
    Font{
        name: "Consolas v5.53"
    }
];
// Indices corresponding to the default font, in this case JB Mono @ Regular
pub const DEFAULT_FONT_INDEX: usize = 0;
pub const DEFAULT_FONT_VARIANT_INDEX: usize = 3;

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
        oversample_h: 3,
        oversample_v: 3,
        ..FontConfig::default()
    }
}
