//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct MultiArg {
    set: (),
    name: String,
    value: String,
}

pub fn options() -> OptionParser<Options> {
    let set = long("set").req_flag(());
    let name = positional("ARG");
    let value = positional("ARG");
    let multi_arg = construct!(MultiArg { set, name, value })
        .anywhere()
        .optional();

    let turbo = long("turbo").switch();
    construct!(Options { multi_arg, turbo }).to_options()
}
