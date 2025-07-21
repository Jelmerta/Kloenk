use std::env;
use std::path::PathBuf;

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = env::var("OUT_DIR")
        .map(|out_dir| {
            PathBuf::from(out_dir) // target/debug or target/release
                .ancestors()
                .nth(3)
                .unwrap()
                .to_path_buf()
        })
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
        .join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}
