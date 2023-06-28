<details><summary><tt>examples/dd.rs</tt></summary>

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

</details>

<details><summary>Output</summary>

`bpaf` generates usual help message with


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-m</b></tt>] [<tt><b>if=</b></tt><tt><i>FILE</i></tt>] [<tt><b>of=</b></tt><tt><i>FILE</i></tt>] [<tt><b>bs=</b></tt><tt><i>SIZE</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-m</b></tt>, <tt><b>--magic</b></tt></dt>
<dd>a usual switch still works</dd>
<dt><tt><b>if=</b></tt><tt><i>FILE</i></tt></dt>
<dd>read from FILE</dd>
<dt></dt>
<dd>[default: -]</dd>
<dt><tt><b>of=</b></tt><tt><i>FILE</i></tt></dt>
<dd>write to FILE</dd>
<dt></dt>
<dd>[default: -]</dd>
<dt><tt><b>bs=</b></tt><tt><i>SIZE</i></tt></dt>
<dd>read/write SIZE blocks at once</dd>
<dt></dt>
<dd>[default: 512]</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


Unlike usual application `dd` takes it arguments in shape of operations
`KEY=VAL` without any dashes, plus usual `--help` and `--version` flags.

To handle that we define custom basic parsers that make handling such operations easy


<div class='bpaf-doc'>
$ app if=/dev/zero of=/tmp/blob bs=1024<br>
Options { magic: false, in_file: "/dev/zero", out_file: "/tmp/blob", block_size: 1024 }
</div>

</details>