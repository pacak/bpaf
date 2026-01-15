//
use bpaf::{doc::*, *};
//
use std::str::FromStr;
// This example is still technically derive API, but derive is limited to gluing
// things together and keeping macro complexity under control.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // `external` here and below derives name from the field name, looking for
    // functions called `block_size`, `count`, etc that produce parsers of
    // the right type.
    // A different way would be to write down the name explicitly:
    // #[bpaf(external(block_size), fallback(1024), display_fallback)]
    #[bpaf(external, fallback(1024), display_fallback)]
    block_size: usize,
    #[bpaf(external, fallback(1))]
    count: usize,
    #[bpaf(external)]
    output_file: String,
    #[bpaf(external)]
    turbo: bool,
}

fn block_size() -> impl Parser<usize> {
    tag("bs=", "BLOCK", "How many bytes to read at once")
}

fn count() -> impl Parser<usize> {
    tag("count=", "NUM", "How many blocks to read")
}

fn output_file() -> impl Parser<String> {
    tag("of=", "FILE", "Save results into this file")
}

fn turbo() -> impl Parser<bool> {
    literal("+turbo")
        .help("Engage turbo mode!")
        .anywhere()
        .map(|_| true)
        .fallback(false)
}

/// Parses a string that starts with `name`, returns the suffix parsed in a usual way
fn tag<T>(name: &'static str, meta: &str, help: impl Into<Doc>) -> impl Parser<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    // closure inside checks if command line argument starts with a given name
    // and if it is - it accepts it, otherwise it behaves like it never saw it
    // it is possible to parse OsString here and strip the prefix with
    // `os_str_bytes` or a similar crate
    any("", move |s: String| Some(s.strip_prefix(name)?.to_owned()))
        // this defines custom metavar for the help message
        // so it looks like something it designed to parse
        .metavar(&[(name, Style::Literal), (meta, Style::Metavar)][..])
        .help(help)
        // this makes it so tag parser tries to read all (unconsumed by earlier parsers)
        // item on a command line instead of trying and failing on the first one
        .anywhere()
        // At this point parser produces `String` while consumer might expect some other
        // type. [`parse`](Parser::parse) handles that
        .parse(|s| s.parse())
}
