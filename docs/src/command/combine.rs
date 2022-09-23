//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cmd {
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    flag: bool,
    cmd: Cmd,
}

fn cmd() -> impl Parser<Cmd> {
    let flag = long("flag")
        .help("This flag is specific to command")
        .switch();
    let arg = long("arg").argument::<usize>("ARG");
    construct!(Cmd { flag, arg })
        .to_options()
        .descr("Command to do something")
        .command("cmd")
        .help("Command to do something")
}

pub fn options() -> OptionParser<Options> {
    let flag = long("flag")
        .help("This flag is specific to the outer layer")
        .switch();
    construct!(Options { flag, cmd() }).to_options()
}
