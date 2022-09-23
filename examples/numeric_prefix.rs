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
