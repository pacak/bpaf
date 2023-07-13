/// You can parse multiple positional elements with earlier being optional as well
/// This example takes two - optional numeric prefix and a command name:
///
/// > numeric_prefix 8 work
/// Options { prefix: Some(8), command: "work" }
///
/// > numeric_prefix sleep
/// Options { prefix: None, command: "sleep" }
///
/// Generated usage reflects that:
/// Usage: numeric_prefix [PREFIX] COMMAND
use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    prefix: Option<usize>,
    command: String,
}

pub fn options() -> OptionParser<Options> {
    let prefix = positional::<usize>("PREFIX")
        .help("Optional numeric command prefix")
        .optional()
        .catch();
    let command = positional::<String>("COMMAND").help("Required command name");

    construct!(Options { prefix, command }).to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
