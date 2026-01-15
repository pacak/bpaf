//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    point: Vec<Point>,
    rotate: bool,
}

#[derive(Debug, Clone)]
struct Point {
    point: (),
    x: usize,
    y: usize,
    z: f64,
}

fn point() -> impl Parser<Point> {
    let point = short('p')
        .long("point")
        .help("Point coordinates")
        .req_flag(());
    let x = positional::<usize>("X").help("X coordinate of a point");
    let y = positional::<usize>("Y").help("Y coordinate of a point");
    let z = positional::<f64>("Z").help("Height of a point above the plane");
    construct!(Point { point, x, y, z }).adjacent()
}

pub fn options() -> OptionParser<Options> {
    let rotate = short('r')
        .long("rotate")
        .help("Face the camera towards the first point")
        .switch();
    let point = point().many();
    construct!(Options { point, rotate }).to_options()
}
