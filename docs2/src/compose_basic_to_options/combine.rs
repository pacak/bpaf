//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
}

pub fn options() -> OptionParser<Options> {
    let argument = short('i').argument::<u32>("ARG");
    construct!(Options { argument })
        .to_options()
        .version("3.1415")
        .descr("This is a short description")
        .header("It can contain multiple blocks, this block goes before options")
        .footer("This one goes after")
}
