use crate::config::compile_time::config_config::*;

/// # Config
/// This module contains submodules that contain structs for configuring the app
///
/// [compile_time] is for compile-time config
///
/// [init_time] is for config that is used whenever the app starts up and initialises, so the app needs to be restarted for the changes to take effect
///
/// [run_time] contains config that can be changed easily at runtime
pub mod compile_time;
pub mod init_time;
pub mod run_time;
use crate::config::init_time::InitTimeAppConfig;
use crate::helper::file_helper::app_current_directory;
use color_eyre::eyre::{Result as Res, WrapErr};
use color_eyre::{Help, SectionExt};
use serde::{Deserialize, Serialize};
use crate::config::run_time::RuntimeAppConfig;

/// Type alias for easily passing around an [AppConfig] struct
pub type Config = &'static mut AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig{
    pub init: InitTimeAppConfig,
    pub runtime: RuntimeAppConfig
}

pub fn load_config() -> Res<AppConfig> {
    // can't use tracing here since it don't exist yet :(

    //load up the file
    let config_path = app_current_directory()?
        .join(BASE_CONFIG_PATH);
    let data = std::fs::read_to_string(config_path)
        .wrap_err_with(|| format!("could not read init config file at {config_path:?}"))?;
    let config = ron::from_str::<AppConfig>(&data)
        .wrap_err("failed to deserialise config")
        .section(data.header("Config Data"))?;

    Ok(config)
}