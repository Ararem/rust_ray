use crate::build::*;
use shadow_rs::formatcp;

pub const APP_TITLE: &'static str =
    formatcp!("{} v{} - {}", PROJECT_NAME, PKG_VERSION, BUILD_TARGET);
