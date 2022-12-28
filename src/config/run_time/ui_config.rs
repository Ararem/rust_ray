//! # [ui_config]
//!
//! Contains UI configuration fields
use serde::{Deserialize, Serialize};

/// Type alias for the type used by [imgui-rs] for colours
pub type Colour = mint::Vector4<f32>;

// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Value that controls how fast the range for the frame info values is lerped. lower values make a smoother (slower) lerp
    pub frame_info_range_smooth_speed: f32,
    /// Oversampling font should help improve text rendering at expense of larger font atlas texture.
    /// Personally, I can't tell the difference
    pub font_oversampling: i32,
    /// Colour arrays used for the UI
    pub colours: UiColours,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            frame_info_range_smooth_speed: 0.03,
            font_oversampling: 1,
            colours: UiColours::default(),
        }
    }
}

/// Colour arrays for use with [`imgui`]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UiColours {
    pub good: Colour,
    pub warning: Colour,
    pub error: Colour,
    pub severe_error: Colour,
}

impl Default for UiColours {
    fn default() -> Self {
        Self {
            good: [1.0, 0.82, 0.0, 1.0].into(),
            warning: [1.0, 0.82, 0.0, 1.0].into(),
            error: [1.0, 0.47, 0.0, 1.0].into(),
            severe_error: [1.0, 0.0, 0.0, 1.0].into(),
        }
    }
}
