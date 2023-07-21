<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    crate_name: String,
    feature_name: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Display detailed information")
        .switch();

    let crate_name = positional("CRATE").help("Crate name to use");

    let feature_name = positional("FEATURE")
        .help("Display information about this feature")
        .optional();

    construct!(Options {
        verbose,
        // You must place positional items and commands after
        // all other parsers
        crate_name,
        feature_name
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
    /// Display detailed information
    #[bpaf(short, long)]
    verbose: bool,

    // You must place positional items and commands after
    // all other parsers
    #[bpaf(positional("CRATE"))]
    /// Crate name to use
    crate_name: String,

    #[bpaf(positional("FEATURE"))]
    /// Display information about this feature
    feature_name: Option<String>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Positional items show up in a separate group of arguments if they contain a help message,
otherwise they will show up only in **Usage** part.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>] <tt><i>CRATE</i></tt> [<tt><i>FEATURE</i></tt>]</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>CRATE</i></tt></dt>
<dd>Crate name to use</dd>
<dt><tt><i>FEATURE</i></tt></dt>
<dd>Display information about this feature</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Display detailed information</dd>
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


You can mix positional items with regular items


<div class='bpaf-doc'>
$ app --verbose bpaf<br>
Options { verbose: true, crate_name: "bpaf", feature_name: None }
</div>


And since `bpaf` API expects to have non positional items consumed before positional ones - you
can use them in a different order. In this example `bpaf` corresponds to a `crate_name` field and
`--verbose` -- to `verbose`.


<div class='bpaf-doc'>
$ app bpaf --verbose<br>
Options { verbose: true, crate_name: "bpaf", feature_name: None }
</div>


In previous examples optional field `feature` was missing, this one contains it.


<div class='bpaf-doc'>
$ app bpaf autocomplete<br>
Options { verbose: false, crate_name: "bpaf", feature_name: Some("autocomplete") }
</div>


Users can use `--` to tell `bpaf` to treat remaining items as positionals - this might be
required to handle unusual items.


<div class='bpaf-doc'>
$ app bpaf -- --verbose<br>
Options { verbose: false, crate_name: "bpaf", feature_name: Some("--verbose") }
</div>


<div class='bpaf-doc'>
$ app -- bpaf --verbose<br>
Options { verbose: false, crate_name: "bpaf", feature_name: Some("--verbose") }
</div>


Without using `--` `bpaf` would only accept items that don't start with `-` as positional.


<div class='bpaf-doc'>
$ app --detailed<br>
<b>Error:</b> expected <tt><i>CRATE</i></tt>, got <b>--detailed</b>. Pass <tt><b>--help</b></tt> for usage information
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
$ app --verbose<br>
<b>Error:</b> expected <tt><i>CRATE</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


You can use [`any`] to work around this restriction.
</details>