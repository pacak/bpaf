//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    multi: Vec<Multi>,
    switch: bool,
}

//
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Multi {
    m: (),
    pos: usize,
    flag: bool,
    arg: Option<usize>,
}

/// You can mix all sorts of things inside the adjacent group
fn multi() -> impl Parser<Multi> {
    let m = short('m').req_flag(());
    let pos = positional::<usize>("POS");
    let arg = long("arg").argument::<usize>("ARG").optional();
    let flag = long("flag").switch();
    construct!(Multi { m, arg, flag, pos }).adjacent()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let multi = multi().many();
    construct!(Options { multi, switch }).to_options()
}
