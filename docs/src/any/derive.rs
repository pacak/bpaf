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
    #[bpaf(any("REST"), guard(not_help, "keep help"), many)]
    /// app will pass anything unused to a child process
    rest: Vec<OsString>,
}

fn not_help(s: &OsString) -> bool {
    s != "--help"
}
