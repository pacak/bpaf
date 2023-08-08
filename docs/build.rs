use std::{error::Error, path::PathBuf};

use md_eval::process_directory2;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed={}/data", env!("CARGO_MANIFEST_DIR"));
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    process_directory2(data_dir, std::env::var_os("OUT_DIR").unwrap())
}
