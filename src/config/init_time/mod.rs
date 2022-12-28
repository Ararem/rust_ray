use serde::{Deserialize, Serialize};

/// Initialisation-time configuration options for the app
/// These will be read at startup (can be modified any time, just the changes will not take effect until restart when they are read)

pub mod ui_config;

/// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct InitTimeAppConfig {
    pub ui_config: ui_config::UiConfig
}