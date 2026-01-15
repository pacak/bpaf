<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    release: bool,
    binary: String,
}

pub fn options() -> OptionParser<Options> {
    let release = short('r')
        .long("release")
        .help("Perform actions in release mode")
        .switch();

    let binary = short('b')
        .long("binary")
        .help("Use this binary")
        .argument("BIN");

    construct!(Options { release, binary })
        .to_options()
        .usage("Usage: my_program [--release] [--binary=BIN] ...")
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, usage("Usage: my_program [--release] [--binary=BIN] ..."))]
pub struct Options {
    #[bpaf(short, long)]
    /// Perform actions in release mode
    release: bool,
    #[bpaf(short, long, argument("BIN"))]
    /// Use this binary
    binary: String,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Method `usage` lets you to override the whole usage line


<div class='bpaf-doc'>
$ app --help<br>
<p>Usage: my_program [--release] [--binary=BIN] ...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-r</b></tt>, <tt><b>--release</b></tt></dt>
<dd>Perform actions in release mode</dd>
<dt><tt><b>-b</b></tt>, <tt><b>--binary</b></tt>=<tt><i>BIN</i></tt></dt>
<dd>Use this binary</dd>
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


It doesn't alter parser's behavior otherwise


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>--binary</b></tt>=<tt><i>BIN</i></tt>, pass <tt><b>--help</b></tt> for usage information
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
$ app -r --binary test<br>
Options { release: true, binary: "test" }
</div>

</details>