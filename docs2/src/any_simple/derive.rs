//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Engage the turbo mode
    turbo: bool,
    #[bpaf(any("REST", not_help), many)]
    /// app will pass anything unused to a child process
    rest: Vec<OsString>,
}

fn not_help(s: OsString) -> Option<OsString> {
    if s == "--help" {
        None
    } else {
        Some(s)
    }
}
