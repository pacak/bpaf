//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: Vec<u32>,
    switches: Vec<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .some("want at least one argument");
    let switches = long("switch")
        .help("some switch")
        .req_flag(true)
        .some("want at least one switch");
    construct!(Options { argument, switches }).to_options()
}
