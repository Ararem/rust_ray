use std::env;
use std::path::PathBuf;

use color_eyre::{eyre, Report};
use tracing::trace;

use crate::config::resources_config::*;

pub fn get_main_resource_folder_path() -> eyre::Result<PathBuf> {
    // Directory of the .exe
    let current_dir = match env::current_exe() {
        Err(error) => Err(Report::new(error).wrap_err("was not able to find current process file location")),
        //Have to convert, since `.parent()` returns a [Path] (reference) not owned variable, so won't live after `exe_path` goes out of scope
        Ok(exe_path) => Ok(exe_path.parent().map(|path_slice| PathBuf::from(path_slice))),
    }.map(
        // maps the returned Ok() value of `exe_path.parent()`
        |maybe_dir| {
            match maybe_dir {
                None => Err(Report::msg("could not get parent (directory) of executable: `exe_path.parent()` returned [None]")),
                Some(dir) => Ok(dir),
            }
        }
    )??;
    //Propagate errors
    trace!("current directory is {current_dir:?}");

    //Add the fonts path onto the base path
    return Ok(current_dir.join(RESOURCES_PATH));
}