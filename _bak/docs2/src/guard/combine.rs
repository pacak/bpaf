//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").guard(
        |n| *n <= 10,
        "Values greater than 10 are only available in the DLC pack!",
    );
    construct!(Options { number }).to_options()
}
