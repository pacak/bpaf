//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    rest: Vec<OsString>,
}

pub fn options() -> OptionParser<Options> {
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let rest = any("REST").os().many();
    construct!(Options { turbo, rest }).to_options()
}
