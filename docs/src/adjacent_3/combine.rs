//
use std::ffi::OsString;
//
use bpaf::*;
//
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    exec: Vec<OsString>,
}

fn exec() -> impl Parser<Vec<OsString>> {
    let start = long("exec").req_flag(());
    let body = any::<OsString>("EXEC")
        .guard(|s| s != ";", "end marker")
        .many()
        .catch();
    let end = any::<OsString>("TAIL").guard(|s| s == ";", "end marker");
    construct!(start, body, end).adjacent().map(|x| x.1)
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let exec = exec();
    construct!(Options { exec, switch }).to_options()
}
