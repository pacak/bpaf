//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    age: usize,
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    let age = long("age")
        .help("Specify user age")
        // but often rust can figure it out from the context,
        // here age is going to be `usize`
        .argument("AGE")
        .fallback(18)
        .display_fallback();

    construct!(Options { name, age }).to_options()
}
