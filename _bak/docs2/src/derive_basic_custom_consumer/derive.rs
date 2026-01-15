//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, switch)]
    switch: bool,

    /// Custom number
    #[bpaf(positional("NUM"))]
    argument: usize,
}
