//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, optional)]
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(anywhere)]
//
#[allow(dead_code)]
pub struct MultiArg {
    #[bpaf(long)]
    set: (),
    #[bpaf(positional)]
    name: String,
    #[bpaf(positional)]
    value: String,
}
