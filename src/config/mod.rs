use crate::config::compile_time::config_config::*;
use ron;
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
use crate::config::run_time::RuntimeAppConfig;

pub fn load_init_config() -> Res<InitTimeAppConfig> {
    // can't use tracing here since it don't exist yet :(

    //load up the file
    let config_path = app_current_directory()?
        .join(BASE_CONFIG_PATH)
        .join(INIT_CONFIG_PATH);
    let data = std::fs::read_to_string(config_path)
        .wrap_err_with(|| format!("could not read init config file at {config_path:?}"))?;
    let config = ron::from_str::<InitTimeAppConfig>(&data)
        .wrap_err("failed to deserialise config")
        .section(data.header("Config Data"))?;

    Ok(config)
}

pub fn load_runtime_config() -> Res<RuntimeAppConfig>{
    let config_path = app_current_directory()?
        .join(BASE_CONFIG_PATH)
        .join(RUN_CONFIG_PATH);
    let data = std::fs::read_to_string(config_path)
        .wrap_err_with(|| format!("could not read runtime config file at {config_path:?}"))?;
    let config = ron::from_str::<RuntimeAppConfig>(&data)
        .wrap_err("failed to deserialise config")
        .section(data.header("Config Data"))?;

    Ok(config)
}