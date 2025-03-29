use anyhow::{Ok, Result, anyhow};
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=assets");
    let out_dir = env::var("OUT_DIR")?;
    let target_dir = PathBuf::from(out_dir) // target/debug or target/release
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["assets/"];
    copy_items(&paths_to_copy, &target_dir, &copy_options)
        .map_err(|e| anyhow!("Failed to copy items: {:?}", e))?;

    #[cfg(target_arch = "wasm32")]
    {
        let paths_to_copy = vec!["web"];
        copy_items(&paths_to_copy, &target_dir, &copy_options)
            .map_err(|e| anyhow!("Failed to copy items: {:?}", e))?;
    }

    Ok(())
}
