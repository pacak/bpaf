//
use bpaf::*;
fn twice_the_num(n: u32) -> u32 {
    n * 2
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
//
#[allow(dead_code)]
pub struct Options {
    #[bpaf(argument::<u32>("N"), map(twice_the_num))]
    number: u32,
}
