use shadow_rs::formatcp;

use crate::build::*;

pub const APP_TITLE: &'static str =
    formatcp!("{} v{} - {}", PROJECT_NAME, PKG_VERSION, BUILD_TARGET);
pub const IMGUI_LOG_FILE_PATH: &'static str = r"./imgui_log.log";
pub const IMGUI_SETTINGS_FILE_PATH: &'static str = r"./imgui_settings.ini";
