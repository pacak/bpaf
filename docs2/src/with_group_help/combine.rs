use bpaf::doc::*;
use bpaf::*;
//
#[allow(dead_code)]
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

fn generate_rectangle_help(meta: MetaInfo) -> Doc {
    let mut buf = Doc::default();
    buf.text("The app takes a rectangle defined by width and height\n\nYou can customize the screen size using ");
    buf.meta(meta, true);
    buf.text(" parameters");
    buf
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
    let rectangle =
        construct!(Rectangle { width, height }).with_group_help(generate_rectangle_help);

    construct!(Options {
        argument,
        rectangle
    })
    .to_options()
}
