//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub struct Rectangle {
    /// Width of the rectangle
    #[bpaf(argument("W"), fallback(10))]
    width: u32,
    /// Height of the rectangle
    #[bpaf(argument("H"), fallback(10))]
    height: u32,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// secret switch
    #[bpaf(external, group_help("Takes a rectangle"))]
    rectangle: Rectangle,
}
