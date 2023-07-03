//! This is not a typical bpaf usage,
//! but you should be able to replicate command line used by dd
use bpaf::{any, construct, doc::Style, short, OptionParser, Parser};
use std::str::FromStr;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    magic: bool,
    in_file: String,
    out_file: String,
    block_size: usize,
}

/// Parses a string that starts with `name`, returns the suffix parsed in a usual way
fn tag<T>(name: &'static str, meta: &str, help: &'static str) -> impl Parser<T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    // it is possible to parse OsString here and strip the prefix with
    // `os_str_bytes` or a similar crate
    any("", move |s: String| Some(s.strip_prefix(name)?.to_owned()))
        // this defines custom metavar for the help message
        .metavar(&[(name, Style::Literal), (meta, Style::Metavar)][..])
        .help(help)
        .anywhere()
        .parse(|s| s.parse())
}

fn in_file() -> impl Parser<String> {
    tag::<String>("if=", "FILE", "read from FILE")
        .fallback(String::from("-"))
        .display_fallback()
}

fn out_file() -> impl Parser<String> {
    tag::<String>("of=", "FILE", "write to FILE")
        .fallback(String::from("-"))
        .display_fallback()
}

fn block_size() -> impl Parser<usize> {
    // it is possible to parse notation used by dd itself as well,
    // using usuze only for simplicity
    tag::<usize>("bs=", "SIZE", "read/write SIZE blocks at once")
        .fallback(512)
        .display_fallback()
}

pub fn options() -> OptionParser<Options> {
    let magic = short('m')
        .long("magic")
        .help("a usual switch still works")
        .switch();
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
