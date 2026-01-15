//
use bpaf::{doc::*, *};
//
use std::str::FromStr;
#[derive(Debug, Clone)]
pub struct Options {
    block_size: usize,
    count: usize,
    output_file: String,
    turbo: bool,
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

pub fn options() -> OptionParser<Options> {
    let block_size = tag("bs=", "BLOCK", "How many bytes to read at once")
        .fallback(1024)
        .display_fallback();
    let count = tag("count=", "NUM", "How many blocks to read").fallback(1);
    let output_file = tag("of=", "FILE", "Save results into this file");

    // this consumes literal value of "+turbo" locate and produces `bool`
    let turbo = literal("+turbo")
        .help("Engage turbo mode!")
        .anywhere()
        .map(|_| true)
        .fallback(false);

    construct!(Options {
        block_size,
        count,
        output_file,
        turbo
    })
    .to_options()
}
