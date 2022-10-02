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
    // in many cases rustc is able to deduct exact type for the argument
    // you are trying to consume, alternatively you can always specify it
    // with turbofish to `argument:`
    // let shorty = short('s').argument::<u64>("ARG");
    let shorty = short('s').argument("ARG");
    construct!(Options { value, shorty }).to_options()
}
