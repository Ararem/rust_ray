use crate::config::compile_time::config_config::*;
use std::fs;
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
use crate::FallibleFn;
use color_eyre::eyre::{Result as Res, WrapErr};
use color_eyre::{Help, SectionExt};
use lazy_static::lazy_static;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub init: InitTimeAppConfig,
    pub runtime: RuntimeAppConfig,
}

/// Attempts to save the currently loaded config to disk
pub fn save_config_to_disk() -> FallibleFn {
    let config_path = app_current_directory()?.join(BASE_CONFIG_PATH);
    let config = read_config_value(|config| config.clone());

    let serialised = to_string_pretty(&config, PrettyConfig::default().separate_tuple_members(true).enumerate_arrays(true)).wrap_err("couldn't serialise config")?;

    fs::write(config_path, serialised).wrap_err("couldn't save serialised config to file")?;

    Ok(())
}

/// Loads the config from disk, if possible
pub fn load_config_from_disk() -> FallibleFn {
    let new_config = fallible_get_disk_config().wrap_err("could not load config from disk")?;
    update_config(|config_ref| *config_ref = new_config);
    Ok(())
}

/// Internal function that tries to get the config from disk. Can fail (and if so returns the error instead)
fn fallible_get_disk_config() -> Res<AppConfig> {
    //load up the file
    let config_path = app_current_directory()?.join(BASE_CONFIG_PATH);
    let data = fs::read_to_string(&config_path).wrap_err_with(|| format!("could not read init config file at {config_path:?}"))?;
    let config = ron::from_str::<AppConfig>(&data).wrap_err("failed to deserialise config").section(data.header("Config Data"))?;

    Ok(config)
}
lazy_static! {
    static ref CONFIG_INSTANCE: Mutex<AppConfig> = Mutex::new(
    {
        // Again, we can't using [tracing] so we gotta use println (ew)
        let maybe_config = fallible_get_disk_config();
        match maybe_config {
            Ok(config) => config,
            Err(report) => {
            let report =
                report.wrap_err("using default font config (could not load config from file)");
            eprintln!("problem loading config: {:?}", report);
            AppConfig::default()
            }
        }
    }
    );
}

/// Reads a config value from the global [AppConfig], and returns it
///
/// # Safety
/// Completely threadsafe.
///
/// This should be slightly faster than [update_config_value] since it runs the function on a copy of the data, unlocking the mutex before the function is called
pub fn read_config_value<T>(func: fn(&AppConfig) -> T) -> T {
    let guard = match CONFIG_INSTANCE.lock() {
        Ok(guard) => guard,
        Err(poison) => {
            // Might recurse if we log warning and then logger tries to access config
            // But i've put a bypass into the log filter so it shouldn't access config for warnings, so this should be fine
            // We definitely can't use any other code though, as that might access config and isn't safe
            warn!(target: GENERAL_WARNING_NON_FATAL, "config instance was poisoned: a thread failed while holding the lock");
            poison.into_inner()
        }
    };

    // Clone so that we can drop the guard and unlock the mutex as soon as possible
    let config: AppConfig = guard.deref().clone();
    drop(guard);

    func(&config)
}

pub fn update_config<T, F: FnOnce(&mut AppConfig) -> T>(func: F) -> T {
    let mut guard = match CONFIG_INSTANCE.lock() {
        Ok(guard) => guard,
        Err(poison) => {
            // Might recurse if we log warning and then logger tries to access config
            // But i've put a bypass into the log filter so it shouldn't access config for warnings, so this should be fine
            // We definitely can't use any other code though, as that might access config and isn't safe
            warn!(target: GENERAL_WARNING_NON_FATAL, "config instance was poisoned: a thread failed while holding the lock");
            poison.into_inner()
        }
    };

    let config = guard.deref_mut();
    let result = func(config);
    drop(guard);
    result
}
