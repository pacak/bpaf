<details><summary>Combinatoric example</summary>

```no_run
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

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
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

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Instead of usual metavariable `any` parsers take something that can represent any value


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>bs=</b></tt><tt><i>BLOCK</i></tt>] [<tt><b>count=</b></tt><tt><i>NUM</i></tt>] <tt><b>of=</b></tt><tt><i>FILE</i></tt> [<tt><b>+turbo</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>bs=</b></tt><tt><i>BLOCK</i></tt></dt>
<dd>How many bytes to read at once</dd>
<dt></dt>
<dd>[default: 1024]</dd>
<dt><tt><b>count=</b></tt><tt><i>NUM</i></tt></dt>
<dd>How many blocks to read</dd>
<dt><tt><b>of=</b></tt><tt><i>FILE</i></tt></dt>
<dd>Save results into this file</dd>
<dt><tt><b>+turbo</b></tt></dt>
<dd>Engage turbo mode!</dd>
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


Output file is required in this parser, other values are optional


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>of=</b></tt><tt><i>FILE</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


<div class='bpaf-doc'>
$ app of=simple.txt<br>
Options { block_size: 1024, count: 1, output_file: "simple.txt", turbo: false }
</div>


Since options are defined with `anywhere` - order doesn't matter


<div class='bpaf-doc'>
$ app bs=10 of=output.rs +turbo<br>
Options { block_size: 10, count: 1, output_file: "output.rs", turbo: true }
</div>


<div class='bpaf-doc'>
$ app +turbo bs=10 of=output.rs<br>
Options { block_size: 10, count: 1, output_file: "output.rs", turbo: true }
</div>




<div class='bpaf-doc'>
$ app bs=65536 count=12 of=hello_world.rs<br>
Options { block_size: 65536, count: 12, output_file: "hello_world.rs", turbo: false }
</div>

</details>