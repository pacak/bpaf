//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, argument("PX"), many, catch)]
    /// Height of a rectangle
    height: Vec<usize>,

    #[bpaf(long("height"), argument("PX"), many, hide)]
    height_str: Vec<String>,

    #[bpaf(long, argument("PX"), many)]
    /// Width of a rectangle
    width: Vec<usize>,

    #[bpaf(long("width"), argument("PX"), many, hide)]
    width_str: Vec<String>,
}
