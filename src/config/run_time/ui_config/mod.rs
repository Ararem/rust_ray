//! # [ui_config]
//!
//! Contains UI configuration fields
use serde::{Deserialize, Serialize};
use frame_info_config::FrameInfoConfig;
use theme::Theme;

mod theme;
mod frame_info_config;
pub mod theme_ext;

// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Oversampling font should help improve text rendering at expense of larger font atlas texture.
    /// Personally, I can't tell the difference
    pub font_oversampling: i32,
    /// Colour arrays used for the UI
    pub colours: Theme,

    pub frame_info: FrameInfoConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_oversampling: 1,
            colours: Theme::default(),
            frame_info: FrameInfoConfig::default(),
        }
    }
}
