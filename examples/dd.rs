//! This is not a typical bpaf usage, but you should be able to replicate command line used by dd

use bpaf::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    magic: bool,
    in_file: String,
    out_file: String,
    block_size: usize,
}

fn tag<T>(name: &'static str, meta: &'static str, help: &'static str) -> impl Parser<T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    // it is possible to parse OsString here and strip the prefix with os_str_bytes or a similar
    // crate
    any::<String>(meta)
        .help(help)
        .parse::<_, _, String>(move |s| match s.strip_prefix(name) {
            None => Err("Wrong tag".to_string()),
            Some(body) => T::from_str(body).map_err(|e| e.to_string()),
        })
        .anywhere()
        .catch()
}

fn in_file() -> impl Parser<String> {
    tag::<String>("if=", "if=FILE", "read from FILE instead of stdin").fallback(String::from("-"))
}

fn out_file() -> impl Parser<String> {
    tag::<String>("of=", "of=FILE", "write to FILE instead of stdout").fallback(String::from("-"))
}

fn block_size() -> impl Parser<usize> {
    // it is possible to parse notation used by dd itself as well, using only ints for simplicity
    tag::<usize>(
        "bs=",
        "bs=SIZE",
        "read/write SIZE blocks at once instead of 512",
    )
    .fallback(512)
}

pub fn options() -> OptionParser<Options> {
    let magic = long("magic").switch();
    construct!(Options {
        magic,
        in_file(),
        out_file(),
        block_size(),
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
