use color_eyre::{eyre, Report};
use std::env;
use std::path::PathBuf;

/// Gets the directory of the current app executable
pub fn app_current_directory() -> eyre::Result<PathBuf> {
    // Directory of the .exe
    match env::current_exe() {
        Err(error) => Err(Report::new(error).wrap_err("was not able to find current process file location")),
        //Have to convert, since `.parent()` returns a [Path] (reference) not owned variable, so won't live after `exe_path` goes out of scope
        Ok(exe_path) => Ok(exe_path.parent().map(PathBuf::from)),
    }.map(
        // maps the returned Ok() value of `exe_path.parent()`
        |maybe_dir| {
            match maybe_dir {
                None => Err(Report::msg("could not get parent (directory) of executable: `exe_path.parent()` returned [None]")),
                Some(dir) => Ok(dir),
            }
        }
    )?
}
