//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
    switch: bool,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .fallback(30);
    let switch = long("switch")
        .help("not that important switch")
        .switch()
        .hide_usage();
    construct!(Options { argument, switch }).to_options()
}
