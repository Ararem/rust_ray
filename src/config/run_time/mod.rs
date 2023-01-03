//! This module defines the configuration struct(s) that configure options for the entire application

pub mod keybindings_config;
pub mod resources_config;
pub mod tracing_config;
pub mod ui_config;

use keybindings_config::*;
use resources_config::ResourcesConfig;
use serde::{Deserialize, Serialize};
use tracing_config::TracingConfig;
use ui_config::UiConfig;

/// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeAppConfig {
    pub keybindings: KeybindingsConfig,
    pub resources: ResourcesConfig,
    pub tracing: TracingConfig,
    pub ui: UiConfig,
}
