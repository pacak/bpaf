use std::{error::Error, path::PathBuf};

fn main() -> std::result::Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={}/data", env!("CARGO_MANIFEST_DIR"));
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    std::fs::create_dir_all(&data_dir)?;
    md_eval::process_directory(data_dir, std::env::var_os("OUT_DIR").unwrap())
}
