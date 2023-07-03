//! How to extract subcommands' args into external structs.

use bpaf::*;

#[derive(Debug, Clone)]
pub struct Foo {
    pub bar: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Foo(Foo),
}

fn main() {
    let bar = short('b')
        .long("bar")
        .help("some bar command")
        .argument::<String>("BAR")
        .optional();

    let bar_cmd = construct!(Foo { bar })
        .to_options()
        .descr("This command will try to do foo given a bar argument");

    let opt = bar_cmd
        .command("foo")
        .help("command for doing foo")
        .map(Command::Foo)
        .to_options()
        .run();

    println!("{:#?}", opt);
}
