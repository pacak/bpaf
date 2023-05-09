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
    let body = any::<OsString, _, _>("EXEC", |s| (s != ";").then_some(s))
        .many()
        .catch();
    let end = any::<OsString, _, _>("TAIL", |s| (s == ";").then_some(()));
    construct!(start, body, end).adjacent().map(|x| x.1)
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let exec = exec();
    construct!(Options { exec, switch }).to_options()
}
