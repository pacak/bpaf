//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, many)]
    multi: Vec<Multi>,
    #[bpaf(short)]
    switch: bool,
}

//
#[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Multi {
    m: (),
    #[bpaf(positional("V1"))]
    val_1: usize,
    #[bpaf(positional("V2"))]
    val_2: usize,
    #[bpaf(positional("V3"))]
    val_3: f64,
}
