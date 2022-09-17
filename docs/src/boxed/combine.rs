//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Command {
    A(String),
    B(String),
}

pub fn options() -> OptionParser<Command> {
    let a = positional("A").map(Command::A).to_options().command("a");
    let b = positional("B").map(Command::B).to_options().command("b");
    let sneaky = false;
    let a = if sneaky {
        construct!(a)
    } else {
        let f = fail("command a is not active");
        construct!(f)
    };
    construct!([a, b]).to_options()
}
