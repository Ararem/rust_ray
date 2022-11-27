use std::env;
use std::error::Error;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<(), Box<dyn Error>> {
    //Copy /resources to our output directory
    {
        const RESOURCES_FOLDER_LOCATION: &str = "resources";

        // Re-runs script if any files in res are changed
        println!("cargo:rerun-if-changed={RESOURCES_FOLDER_LOCATION}/*");

        let mut options = CopyOptions::new();
        // Overwrite existing files with same name
        options.overwrite = true;

        let mut from_path = Vec::new();
        from_path.push(RESOURCES_FOLDER_LOCATION);

        let out_path = env::var_os("CARGO_BUILD_TARGET_DIR ").unwrap();

        // copy_items(&from_path, &out_path, &options)?;
    }

    shadow_rs::new()?;

    Ok(())
}