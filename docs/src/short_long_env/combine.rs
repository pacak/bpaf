//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    switch: bool,
    arg: usize,
    username: String,
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s') // first `short` creates a builder
        .short('S') // second switch is a hidden alias
        .long("switch") // visible long name
        .long("also-switch") // hidden alias
        .help("Switch with many names")
        .switch(); // `switch` finalizes the builder

    let arg = long("argument") // long is also a builder
        .short('a')
        .short('A')
        .long("also-arg")
        .help("Argument with names")
        .argument::<usize>("ARG");

    let username = long("user")
        .short('u')
        .env("USER1")
        .help("Custom user name")
        .argument::<String>("USER");

    construct!(Options {
        switch,
        arg,
        username
    })
    .to_options()
}
