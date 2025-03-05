<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    release: bool,
    default_features: bool,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Produce verbose output")
        .switch();
    let release = long("release")
        .help("Build artifacts in release mode")
        .flag(true, false);
    let default_features = long("no-default-features")
        .help("Do not activate default features")
        // default_features uses opposite values,
        // producing `true` when value is absent
        .flag(false, true);

    construct!(Options {
        verbose,
        release,
        default_features,
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
    /// Produce verbose output
    // bpaf uses `switch` for `bool` fields in named
    // structs unless consumer attribute is present.
    // But it is also possible to give it explicit
    // consumer annotation to serve as a reminder:
    // #[bpaf(short, long, switch)]
    #[bpaf(short, long)]
    verbose: bool,

    #[bpaf(flag(true, false))]
    /// Build artifacts in release mode
    release: bool,

    /// Do not activate default features
    // default_features uses opposite values,
    // producing `true` when value is absent
    #[bpaf(long("no-default-features"), flag(false, true))]
    default_features: bool,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In `--help` output `bpaf` shows switches as usual flags with no meta variable attached


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>] [<tt><b>--release</b></tt>] [<tt><b>--no-default-features</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Produce verbose output</dd>
<dt><tt><b>    --release</b></tt></dt>
<dd>Build artifacts in release mode</dd>
<dt><tt><b>    --no-default-features</b></tt></dt>
<dd>Do not activate default features</dd>
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


Both `switch` and `flag` succeed if value is not present, `switch` returns `false`, `flag` returns
second value.


<div class='bpaf-doc'>
$ app <br>
Options { verbose: false, release: false, default_features: true }
</div>


When value is present - `switch` returns `true`, `flag` returns first value.


<div class='bpaf-doc'>
$ app --verbose --no-default-features --detailed<br>
<b>Error:</b> <b>--detailed</b> is not expected in this context
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


Like with most parsrs unless specified `switch` and `flag` consume at most one item from the
command line:


<div class='bpaf-doc'>
$ app --no-default-features --no-default-features<br>
<b>Error:</b> argument <tt><b>--no-default-features</b></tt> cannot be used multiple times in this context
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

</details>