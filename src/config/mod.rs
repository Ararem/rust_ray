use crate::config::compile_time::config_config::*;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

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
use crate::config::run_time::RuntimeAppConfig;
use crate::helper::file_helper::app_current_directory;
use crate::helper::logging::event_targets::*;
use color_eyre::eyre::{Result as Res, WrapErr};
use color_eyre::{Help, Report, SectionExt};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Type alias for easily passing around an [AppConfig] struct
pub type Config = &'static mut AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct AppConfig {
    pub init: InitTimeAppConfig,
    pub runtime: RuntimeAppConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            init: InitTimeAppConfig::default(),
            runtime: RuntimeAppConfig::default(),
        }
    }
}

fn load_config() -> Res<AppConfig> {
    // can't use tracing here since it don't exist yet :(

    //load up the file
    let config_path = app_current_directory()?.join(BASE_CONFIG_PATH);
    let data = std::fs::read_to_string(&config_path)
        .wrap_err_with(|| format!("could not read init config file at {config_path:?}"))?;
    let config = ron::from_str::<AppConfig>(&data)
        .wrap_err("failed to deserialise config")
        .section(data.header("Config Data"))?;

    Ok(config)
}
lazy_static! {
    static ref CONFIG_INSTANCE: Mutex<Option<&'static mut AppConfig>> = Mutex::new(None);
}

/// Reads a config value from the global [AppConfig], and returns it
///
/// # Safety
/// Completely threadsafe.
///
/// This should be slightly faster than [update_config_value] since it runs the function on a copy of the data, unlocking the mutex before the function is called
pub fn read_config_value<T>(func: fn(&AppConfig) -> T) -> T {
    let mut guard = match CONFIG_INSTANCE.lock() {
        Ok(guard) => guard,
        Err(poison) => {
            // Might recurse if we log warning and then logger tries to access config
            // But i've put a bypass into the log filter so it shouldn't access config for warnings, so this should be fine
            // We definitely can't use any other code though, as that might access config and isn't safe
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "config instance was poisoned: a thread failed while holding the lock"
            );
            poison.into_inner()
        }
    };

    // If config isn't loaded, load it
    let config: AppConfig = match guard.deref() {
        Some(cfg) => (*cfg).clone(),
        None => {
            // In the case that we didn't already have an instance loaded, we have to load it
            // We also need to update the singleton now
            let owned_config = match load_config() {
                Ok(conf) => conf,
                Err(err) => {
                    let report = Report::wrap_err(
                        err,
                        "could not load config from file, using default config instead",
                    );
                    println!("{:?}", report);
                    AppConfig::default()
                }
            };
            *guard = Some(Box::leak(Box::new(owned_config.clone())));
            owned_config
        }
    };
    drop(guard);

    let result = func(&config);

    result
}

pub fn update_config<T>(func: fn(&mut AppConfig) -> T) -> T {
    let mut guard = match CONFIG_INSTANCE.lock() {
        Ok(guard) => guard,
        Err(poison) => {
            // Might recurse if we log warning and then logger tries to access config
            // But i've put a bypass into the log filter so it shouldn't access config for warnings, so this should be fine
            // We definitely can't use any other code though, as that might access config and isn't safe
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "config instance was poisoned: a thread failed while holding the lock"
            );
            poison.into_inner()
        }
    };

    let result = match guard.deref_mut() {
        // Already have config loaded
        Some(cfg) => {
            func(*cfg)
        },
        // Don't have config loaded
        None => {
            // In the case that we didn't already have an instance loaded, we have to load it
            let mut owned_config = match load_config() {
                Ok(conf) => conf,
                Err(err) => {
                    let report = Report::wrap_err(
                        err,
                        "could not load config from file, using default config instead",
                    );
                    println!("{:?}", report);
                    AppConfig::default()
                }
            };
            let temp = func(&mut owned_config);
            *guard = Some(Box::leak(Box::new(owned_config))); //Update the singleton with the instance we just loaded
            temp
        }
    };

    drop(guard);

    result
}
