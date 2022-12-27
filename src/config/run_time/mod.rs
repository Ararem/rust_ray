//! This module defines the configuration struct(s) that configure options for the entire application

pub mod keybindings_config;
pub mod resources_config;
pub mod tracing_config;
pub mod ui_config;

use crate::config::run_time::resources_config::ResourcesConfig;
use crate::config::run_time::tracing_config::TracingConfig;
use keybindings_config::*;
use serde::{Deserialize, Serialize};

/// Base configuration struct that contains options that configure the entire app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAppConfig {
    pub keybindings: KeybindingsConfig,
    pub resources: ResourcesConfig,
    pub tracing: TracingConfig,
}
