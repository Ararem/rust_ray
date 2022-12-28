use std::path::PathBuf;

use crate::config::read_config_value;
use color_eyre::eyre;

use crate::helper::file_helper::app_current_directory;

pub fn get_main_resource_folder_path() -> eyre::Result<PathBuf> {
    Ok(app_current_directory()?.join(read_config_value(|config| {
        config.runtime.resources.resources_path.clone()
    })))
}
