//! A simple combinatoric parser with extra metainformation attached, see --help

use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
struct Out {
    rect: Option<Rect>,
    verbose: bool,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
struct Rect {
    width: usize,
    height: usize,
}

fn main() {
    let width = short('w')
        .long("width")
        .help("Width of the rectangle")
        .argument::<usize>("PX");

    let height = short('h')
        .long("height")
        .help("Height of the rectangle")
        .argument::<usize>("PX");

    let rect = construct!(Rect { width, height })
        .group_help("Rectangle is defined by width and height in meters")
        .optional();

    let verbose = short('v')
        .long("verbose")
        .help("Print computation steps")
        .switch();

    let opt = construct!(Out { verbose, rect })
        .to_options()
        .descr("This program calculates rectangle's area")
        .header("vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv")
        .footer("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^")
        .run();
    println!("{:#?}", opt);
}
