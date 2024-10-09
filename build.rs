use anyhow::{anyhow, Ok, Result};
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    // This tells Cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=resources/*");

    let out_dir = env::var("OUT_DIR")?;
    println!("{:?}", env::var("OUT_DIR"));
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["resources/"];
    copy_items(&paths_to_copy, out_dir, &copy_options)
        .map_err(|e| anyhow!("Failed to copy items: {:?}", e))?;

    Ok(())
}
