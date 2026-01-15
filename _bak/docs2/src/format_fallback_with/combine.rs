//
use bpaf::*;
use std::{fmt::Display as _, path::PathBuf};
fn try_to_get_log_file() -> Result<PathBuf, &'static str> {
    Ok(PathBuf::from("logfile.txt"))
}

#[derive(Debug, Clone)]
pub struct Options {
    log_file: PathBuf,
}

pub fn options() -> OptionParser<Options> {
    let log_file = long("log-file")
        .help("Path to log file")
        .argument::<PathBuf>("FILE")
        .guard(
            |log_file| !log_file.is_dir(),
            "The log file can't be a directory",
        )
        .fallback_with(try_to_get_log_file)
        .format_fallback(|path, f| path.display().fmt(f));
    construct!(Options { log_file }).to_options()
}
