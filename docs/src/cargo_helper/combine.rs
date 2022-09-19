//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: usize,
    switch: bool,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("An argument")
        .argument::<usize>("ARG");
    let switch = short('s').help("A switch").switch();
    let options = construct!(Options { argument, switch });

    cargo_helper("pretty", options).to_options()
}
