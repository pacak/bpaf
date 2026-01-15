use bpaf::*;

#[derive(Debug, Clone)]
pub struct Options {
    message: String,
}

pub fn options() -> OptionParser<Options> {
    let message = positional("MESSAGE").help("Message to print in a big friendly letters");
    construct!(Options { message }).to_options()
}
