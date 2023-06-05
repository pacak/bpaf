```no_run
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

```
<details>
<summary style="display: list-item;">Examples</summary>


`dd` takes parameters in `name=value` shape
```console
% app if=/dev/zero of=file bs=10000
Options { magic: false, in_file: "/dev/zero", out_file: "file", block_size: 10000 }
```

Usual properties about ordering holds, you can also mix them with regular options
```console
% app if=/dev/zero of=file bs=10000 --magic
Options { magic: true, in_file: "/dev/zero", out_file: "file", block_size: 10000 }
```

Fallback works as expected
```console
% app --magic bs=42
Options { magic: true, in_file: "-", out_file: "-", block_size: 42 }
```

Best effort to generate help, but you can always customize it further
```console
% app --help
Usage: [--magic] [<if=FILE>] [<of=FILE>] [<bs=SIZE>]

Available options:
        --magic
    <if=FILE>    read from FILE instead of stdin
    <of=FILE>    write to FILE instead of stdout
    <bs=SIZE>    read/write SIZE blocks at once instead of 512
    -h, --help   Prints help information
```

</details>
