use std::{error::Error, path::PathBuf};

use md_eval::process_directory;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=data");
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    process_directory(data_dir, std::env::var_os("OUT_DIR").unwrap())
}
