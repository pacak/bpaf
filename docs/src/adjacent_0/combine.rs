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
    val_1: usize,
    val_2: usize,
    val_3: f64,
}

fn multi() -> impl Parser<Multi> {
    let m = short('m').req_flag(());
    let val_1 = positional::<usize>("V1");
    let val_2 = positional::<usize>("V2");
    let val_3 = positional::<f64>("V3");
    construct!(Multi {
        m,
        val_1,
        val_2,
        val_3
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let multi = multi().many();
    construct!(Options { multi, switch }).to_options()
}
