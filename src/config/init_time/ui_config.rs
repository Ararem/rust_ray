use serde::{Deserialize, Serialize};
use winit::dpi::{LogicalSize, Size};

/// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    /// Whether the main OS window should start maximised (when created initially)
    pub start_maximised: bool,
    /// flag for if the renderer should enable VSync
    ///
    /// see [glutin::ContextBuilder::with_vsync]
    pub vsync: bool,
    /// Whether hardware acceleration is required to be a certain value ([Some]), or automatic ([None])
    pub hardware_acceleration: Option<bool>,
    ///Sets the multisampling level to request. A value of 0 indicates that multisampling must not be enabled.
    ///
    /// Must be a power of 2
    pub multisampling: u16,
}

impl std::default::Default for UiConfig {
    fn default() -> Self {
        Self {
            start_maximised: true,
            vsync: false,
            hardware_acceleration: Some(true),
            multisampling: 2,
        }
    }
}
