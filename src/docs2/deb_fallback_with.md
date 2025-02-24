<details><summary>Combinatoric example</summary>

```no_run
fn try_to_get_version() -> Result<usize, &'static str> {
    Ok(42)
}

#[derive(Debug, Clone)]
pub struct Options {
    version: usize,
}

pub fn options() -> OptionParser<Options> {
    let version = long("version")
        .help("Specify protocol version")
        .argument("VERS")
        .fallback_with(try_to_get_version)
        .debug_fallback();
    construct!(Options { version }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
fn try_to_get_version() -> Result<usize, &'static str> {
    Ok(42)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("VERS"), fallback_with(try_to_get_version), debug_fallback)]
    /// Specify protocol version
    version: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`fallback_with` changes parser to fallback to a value that comes from a potentially failing
computation when argument is not specified


<div class='bpaf-doc'>
$ app <br>
Options { version: 42 }
</div>


If value is present - fallback value is ignored


<div class='bpaf-doc'>
$ app --version 10<br>
Options { version: 10 }
</div>


Parsing errors are preserved and presented to the user


<div class='bpaf-doc'>
$ app --version ten<br>
<b>Error:</b> couldn't parse <b>ten</b>: invalid digit found in string
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


`bpaf` encases parsers with fallback value of some sort in usage with `[]`


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--version</b></tt>=<tt><i>VERS</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --version</b></tt>=<tt><i>VERS</i></tt></dt>
<dd>Specify protocol version</dd>
<dt></dt>
<dd>[default: 42]</dd>
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

</details>