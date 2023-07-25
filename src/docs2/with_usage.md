
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
        .with_usage(|u| {
            let mut doc = Doc::default();
            doc.emphasis("Usage: ");
            doc.literal("my_program");
            doc.text(" ");
            doc.doc(&u);
            doc
        })
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

`with_usage` lets you to place some custom text around generated usage line


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage: </b><tt><b>my_program</b></tt> [<tt><b>-r</b></tt>] <tt><b>-b</b></tt>=<tt><i>BIN</i></tt></p><p><div>
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