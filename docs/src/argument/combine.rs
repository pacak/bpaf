//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    value: isize,
    shorty: u64,
}

pub fn options() -> OptionParser<Options> {
    let value = long("value").argument::<isize>("ARG").fallback(100);
    let shorty = short('s').argument::<u64>("ARG");
    construct!(Options { value, shorty }).to_options()
}
