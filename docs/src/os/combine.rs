//
use std::ffi::OsString;
//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    arg: OsString,
    pos: Option<OsString>,
}

pub fn options() -> OptionParser<Options> {
    let arg = long("arg").help("consume a String").argument("ARG").os();
    let pos = positional("POS")
        .help("consume an OsString")
        .os()
        .optional();

    construct!(Options { arg, pos }).to_options()
}
