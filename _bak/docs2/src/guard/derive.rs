//
use bpaf::*;
fn dlc_check(number: &u32) -> bool {
    *number <= 10
}

const DLC_NEEDED: &str = "Values greater than 10 are only available in the DLC pack!";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("N"), guard(dlc_check, DLC_NEEDED))]
    number: u32,
}
