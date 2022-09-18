//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(fallback(100))]
    value: isize,
    #[bpaf(short)]
    shorty: u64,
}
