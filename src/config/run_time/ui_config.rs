//! # [ui_config]
//!
//! Contains UI configuration fields
use serde::{Deserialize, Serialize};

/// Type alias for the type used by [imgui-rs] for colours
pub type Colour = mint::Vector4<f32>;

// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Oversampling font should help improve text rendering at expense of larger font atlas texture.
    /// Personally, I can't tell the difference
    pub font_oversampling: i32,
    /// Colour arrays used for the UI
    pub colours: UiColours,

    pub frame_info: FrameInfoConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_oversampling: 1,
            colours: UiColours::default(),
            frame_info: FrameInfoConfig::default(),
        }
    }
}

/// Colour arrays for use with [`imgui`]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UiColours {}

impl Default for UiColours {
    fn default() -> Self {
        Self {
            // Normal text colours
            normal: [1.0, 1.0, 1.0, 1.0].into(),
            subtle: [0.8, 0.8, 0.8, 1.0].into(),
            accent: [0.8, 0.8, 0.8, 1.0].into(),

            // Tracing/logging colours
            trace: [1.0, 0.0, 0.0, 1.0].into(),
            debug: [1.0, 0.0, 0.0, 1.0].into(),
            info: [1.0, 0.0, 0.0, 1.0].into(),
            warn: [1.0, 0.0, 0.0, 1.0].into(),
            error: [1.0, 0.0, 0.0, 1.0].into(),

            // Severity
            good: [1.0, 0.0, 0.0, 1.0].into(),
            neutral: [1.0, 0.82, 0.0, 1.0].into(),
            bad: [1.0, 0.3, 0.0, 1.0].into(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct FrameInfoConfig {
    /// Size of the 'chunks' used when averaging frame values
    pub chunked_average_smoothing_size: usize,
    /// Toggle for if the minimum value shown should always be zero
    pub min_always_at_zero: bool,
    /// The number of frames of information to display
    pub num_frames_to_display: usize,
    /// The number of frames of information to record. This value will be clamped to a max of [MAX_FRAMES_TO_TRACK][super::super::config::compile_time::ui_config::MAX_FRAMES_TO_TRACK]
    pub num_frames_to_track: usize,
    /// Value that controls how fast the range for the frame info values is lerped. lower values make a smoother (slower) lerp
    pub smooth_speed: f32,
}

impl Default for FrameInfoConfig {
    fn default() -> Self {
        Self {
            chunked_average_smoothing_size: 8,
            min_always_at_zero: true,
            num_frames_to_track: 32_000,
            num_frames_to_display: 1920,
            smooth_speed: 0.03,
        }
    }
}
