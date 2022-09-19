//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options("pretty"))]
pub struct Options {
    /// An argument
    argument: usize,
    /// A switch
    #[bpaf(short)]
    switch: bool,
}
