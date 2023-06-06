//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, many)]
    point: Vec<Point>,
    #[bpaf(short, long)]
    /// Face the camera towards the first point
    rotate: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Point {
    #[bpaf(short, long)]
    /// Point coordinates
    point: (),
    #[bpaf(positional("X"))]
    /// X coordinate of a point
    x: usize,
    #[bpaf(positional("Y"))]
    /// Y coordinate of a point
    y: usize,
    #[bpaf(positional("Z"))]
    /// Height of a point above the plane
    z: f64,
}
