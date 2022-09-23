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
    // You can use FromUtf8 type tag to parse things that only implement `FromStr`, but not `FromOsStr`
    // `u64` implements both and only used as an example
    let shorty = short('s').argument::<FromUtf8<u64>>("ARG");
    construct!(Options { value, shorty }).to_options()
}
