//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    number: u32,
}
pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").map(|x| x * 2);
    construct!(Options { number }).to_options()
}
