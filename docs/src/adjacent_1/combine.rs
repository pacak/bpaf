//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    multi: Vec<Rect>,
}

//
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Rect {
    item: (),
    width: usize,
    height: usize,
    painted: bool,
}

fn multi() -> impl Parser<Rect> {
    let item = long("rect").req_flag(());
    let width = long("width").argument("PX").from_str::<usize>();
    let height = long("height").argument("PX").from_str::<usize>();
    let painted = long("painted").switch();
    construct!(Rect {
        item,
        width,
        height,
        painted,
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let multi = multi().many();
    construct!(Options { multi, switch }).to_options()
}
