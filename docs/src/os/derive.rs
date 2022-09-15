//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    /// consume a String
    arg: String,
    /// consume an OsString
    #[bpaf(positional)]
    pos: Option<OsString>,
}
