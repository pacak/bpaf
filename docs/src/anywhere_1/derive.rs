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
#[bpaf(adjacent)]
//
#[allow(dead_code)]
pub struct MultiArg {
    #[bpaf(long)]
    set: (),
    #[bpaf(positional("NAME"))]
    /// Name for the option
    name: String,
    #[bpaf(positional("VAL"))]
    /// Value to set
    value: String,
}
