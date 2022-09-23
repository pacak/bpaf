//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(fallback(100))]
    value: isize,
    // You can use FromUtf8 type tag to parse things that only implement FromStr, but not FromOsStr
    // `u64` implements both and only used as an example
    #[bpaf(short, argument::<FromUtf8<u64>>("ARG"))]
    shorty: u64,
}
