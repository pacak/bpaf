//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Rectangle {
    width: u32,
    height: u32,
}

//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
    rectangle: Rectangle,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .fallback(30);

    let width = long("width")
        .help("Width of the rectangle")
        .argument("W")
        .fallback(10);
    let height = long("height")
        .help("Height of the rectangle")
        .argument("H")
        .fallback(10);
    let rectangle = construct!(Rectangle { width, height }).group_help("Takes a rectangle");

    construct!(Options {
        argument,
        rectangle
    })
    .to_options()
}
