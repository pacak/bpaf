//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    rect: Vec<Rect>,
    mirror: bool,
}

#[derive(Debug, Clone)]
struct Rect {
    rect: (),
    width: usize,
    height: usize,
    painted: bool,
}

fn rect() -> impl Parser<Rect> {
    let rect = long("rect").help("Define a new rectangle").req_flag(());
    let width = short('w')
        .long("width")
        .help("Rectangle width in pixels")
        .argument::<usize>("PX");
    let height = short('h')
        .long("height")
        .help("Rectangle height in pixels")
        .argument::<usize>("PX");
    let painted = short('p')
        .long("painted")
        .help("Should rectangle be filled?")
        .switch();
    construct!(Rect {
        rect,
        width,
        height,
        painted,
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let mirror = long("mirror").help("Mirror the image").switch();
    let rect = rect().many();
    construct!(Options { rect, mirror }).to_options()
}
