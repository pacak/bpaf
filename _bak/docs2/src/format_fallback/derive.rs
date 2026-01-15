//
use bpaf::*;
use std::{fmt::Display as _, path::PathBuf};
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
pub struct Options {
    /// Path to log file
    #[bpaf(
        argument("FILE"),
        guard(|log_file| !log_file.is_dir(), "The log file can't be a directory"),
        fallback(PathBuf::from("logfile.txt")),
        format_fallback(|path, f| path.display().fmt(f)),
    )]
    log_file: PathBuf,
}
