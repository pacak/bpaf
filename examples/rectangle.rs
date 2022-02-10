use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
struct Out {
    rect: Rect,
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
        .argument("PX")
        .from_str::<usize>();

    let height = short('h')
        .long("height")
        .help("Height of the rectangle")
        .argument("PX")
        .from_str::<usize>();

    let rect = construct!(Rect { width, height })
        .help("Rectangle is defined by width and height in meters");

    let verbose = short('v')
        .long("verbose")
        .help("Print computation steps")
        .switch();

    let parser = construct!(Out { verbose, rect });
    let opt = Info::default()
        .descr("This program calculates rectangle's area")
        .header("vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv")
        .footer("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^")
        .for_parser(parser)
        .run();
    println!("{:#?}", opt);
}
