//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(long, argument("PX"), optional, catch)]
    /// Height of a rectangle
    height: Option<usize>,

    #[bpaf(long("height"), argument("PX"), optional, hide)]
    height_str: Option<String>,

    #[bpaf(long, argument("PX"), optional)]
    /// Width of a rectangle
    width: Option<usize>,

    #[bpaf(long("width"), argument("PX"), optional, hide)]
    width_str: Option<String>,
}
