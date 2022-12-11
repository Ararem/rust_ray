//! # [ui_config]
//!
//! Contains UI configuration fields
//!
//! # Notes:
//! With [VSYNC] and [HARDWARE_ACCELERATION], I've had to hardcode these since changing them makes glutin crap itself:
//! `could not initialise display: GlutinCreationError(NoAvailablePixelFormat)`
use imgui::FontConfig;
use imgui_winit_support::winit::dpi::{LogicalSize, Size};

/// Default value for the size of the main operating system window
pub const DEFAULT_WINDOW_SIZE: Size = Size::Logical(LogicalSize { width: 1600.0, height: 900.0 });
/// Whether the main OS window should start maximised (when created initially)
pub const START_MAXIMIZED: bool = true;
/// flag for if the renderer should enable VSync
///
/// see [glutin::ContextBuilder::with_vsync]
pub const VSYNC: bool = true;
///Sets the multisampling level to request. A value of 0 indicates that multisampling must not be enabled.
///
/// Must be a power of 2
pub const MULTISAMPLING: u16 = 2;
/// Value is [None], meaning hardware acceleration is *not* required
pub const HARDWARE_ACCELERATION: Option<bool> = Some(true);
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
