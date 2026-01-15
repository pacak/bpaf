//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, many)]
    rect: Vec<Rect>,
    /// Mirror the image
    mirror: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Rect {
    /// Define a new rectangle
    rect: (),
    #[bpaf(short, long, argument("PX"))]
    /// Rectangle width in pixels
    width: usize,
    #[bpaf(short, long, argument("PX"))]
    /// Rectangle height in pixels
    height: usize,
    #[bpaf(short, long)]
    /// Should rectangle be filled?
    painted: bool,
}
