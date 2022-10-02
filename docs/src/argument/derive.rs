//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(fallback(100))]
    value: isize,
    // in many cases rustc is able to deduct exact type for the argument
    // you are trying to consume, alternatively you can always specify it
    // with turbofish to `argument:`
    // #[bpaf(short, argument::<u64>("ARG"))]
    #[bpaf(short, argument("ARG"))]
    shorty: u64,
}
