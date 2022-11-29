use std::env;
use std::error::Error;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //Copy /resources to our output directory
    {
        const RESOURCES_FOLDER_LOCATION: &str = "src/resources/app_resources";

        // Re-runs script if any files in res are changed
        println!("cargo:rerun-if-changed={RESOURCES_FOLDER_LOCATION}/*");

        let options = {
            let mut o = CopyOptions::new();
            o.overwrite = true; // Overwrite existing files with same name
            o
        };

        // TODO: Use the path API for this???
        let source_path = format!("{}/{}", env::var("CARGO_MANIFEST_DIR")?, RESOURCES_FOLDER_LOCATION);
        // Have to jump up a three levels because cargo adds some extra directories: "\rust_ray\target\debug\build\rust_ray-e17a28a2c53dbfbd\out"
        let dest_path =  format!("{}/../../../", env::var("OUT_DIR")?);

        p!("src:{source_path}");
        p!("dest:{dest_path}");
        copy_items(&vec![source_path], &dest_path, &options)?;
    }

    shadow_rs::new()?;

    Ok(())
}