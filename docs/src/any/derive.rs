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
    #[bpaf(any("REST"))]
    rest: Vec<OsString>,
}
