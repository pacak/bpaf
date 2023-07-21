<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    binary: String,
    args: Vec<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce detailed report")
        .switch();
    let binary = long("bin").help("Binary to execute").argument("BIN");
    let args = positional("ARG")
        .help("Arguments for the binary")
        .strict()
        .many();
    construct!(Options {
        verbose,
        binary,
        args
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
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Produce detailed report
    verbose: bool,
    #[bpaf(long("bin"), argument("BIN"))]
    /// Binary to execute
    binary: String,
    #[bpaf(positional("ARG"), strict, many)]
    /// Arguments for the binary
    args: Vec<String>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Usage line for a cargo-run like app that takes an app name and possibly many strictly
positional child arguments can look like this:


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>] <tt><b>--bin</b></tt>=<tt><i>BIN</i></tt> <tt><b>--</b></tt> [<tt><i>ARG</i></tt>]...</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>ARG</i></tt></dt>
<dd>Arguments for the binary</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Produce detailed report</dd>
<dt><tt><b>    --bin</b></tt>=<tt><i>BIN</i></tt></dt>
<dd>Binary to execute</dd>
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


Here any argument passed before double dash goes to the parser itself


<div class='bpaf-doc'>
$ app --bin dd --verbose<br>
Options { verbose: true, binary: "dd", args: [] }
</div>


Anything after it - collected into strict arguments


<div class='bpaf-doc'>
$ app --bin dd -- --verbose<br>
Options { verbose: false, binary: "dd", args: ["--verbose"] }
</div>

</details>