//
use bpaf::*;
use std::{fmt::Display as _, path::PathBuf};
fn try_to_get_log_file() -> Result<PathBuf, &'static str> {
    Ok(PathBuf::from("logfile.txt"))
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Path to log file
    #[bpaf(
        argument("FILE"),
        guard(|log_file| !log_file.is_dir(), "The log file can't be a directory"),
        fallback_with(try_to_get_log_file),
        format_fallback(|path, f| path.display().fmt(f)),
    )]
    log_file: PathBuf,
}
