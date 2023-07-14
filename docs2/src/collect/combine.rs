//
use bpaf::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Options {
    argument: BTreeSet<u32>,
    switches: BTreeSet<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .collect();
    let switches = long("switch").help("some switch").switch().collect();
    construct!(Options { argument, switches }).to_options()
}
