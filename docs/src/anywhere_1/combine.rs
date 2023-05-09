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
    let name = positional("NAME").help("Name for the option");
    let value = positional("VAL").help("Value to set");
    let multi_arg = construct!(MultiArg { set, name, value })
        .adjacent()
        .optional();

    let turbo = long("turbo").switch();
    construct!(Options { multi_arg, turbo }).to_options()
}
