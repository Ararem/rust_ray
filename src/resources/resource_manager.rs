use std::path::PathBuf;

use color_eyre::{eyre};
use crate::config::run_time::RuntimeAppConfig;

use crate::helper::file_helper::app_current_directory;

pub fn get_main_resource_folder_path(config: RuntimeAppConfig) -> eyre::Result<PathBuf> {
    let mut current_dir = app_current_directory()?;
    Ok(current_dir.join(config.resources.resources_path))
}
