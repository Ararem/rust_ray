#![allow(unused_imports)]

use crate::build::*;
use shadow_rs::formatcp;

pub const APP_TITLE: &str = formatcp!("{} v{} - {}", PROJECT_NAME, PKG_VERSION, BUILD_TARGET);
pub const IMGUI_LOG_FILE_PATH: &str = r"./imgui_log.log";
pub const IMGUI_SETTINGS_FILE_PATH: &str = r"./imgui_settings.ini";
