//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    banana: bool,
    switch: bool,
}

// accepts `-banana`, note a single dash
fn banana() -> impl Parser<bool> {
    short('b')
        .argument::<std::ffi::OsString>("anana")
        .adjacent()
        .guard(|b| b == "anana", "not anana")
        .optional()
        .catch()
        .map(|b| b.is_some())
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    // banana() is just a syntax construct! allows, not magic
    construct!(Options { banana(), switch }).to_options()
}
