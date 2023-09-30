<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
}

pub fn options() -> OptionParser<Options> {
    let argument = short('i').argument::<u32>("ARG");
    construct!(Options { argument })
        .to_options()
        .version("3.1415")
        .descr("This is a short description")
        .header("It can contain multiple blocks, this block goes before options")
        .footer("This one goes after")
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version("3.1415"))]
/// This is a short description
///
///
/// It can contain multiple blocks, this block goes before options
///
///
/// This one goes after
pub struct Options {
    #[bpaf(short('i'))]
    argument: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In addition to all the arguments specified by user `bpaf` adds a few more. One of them is
`--help`:


<div class='bpaf-doc'>
$ app --help<br>
<p>This is a short description</p><p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-i</b></tt>=<tt><i>ARG</i></tt></p><p>It can contain multiple blocks, this block goes before options</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-i</b></tt>=<tt><i>ARG</i></tt></dt>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
<dt><tt><b>-V</b></tt>, <tt><b>--version</b></tt></dt>
<dd>Prints version information</dd>
</dl>
</p><p>This one goes after</p>
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


The other one is `--version` - passing a string literal or something like
`env!("CARGO_PKG_VERSION")` to get version from `cargo` directly usually works


<div class='bpaf-doc'>
$ app --version<br>
<p>Version: 3.1415</p>
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


Other than that `bpaf` tries its best to provide a helpful error messages


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> expected <tt><b>-i</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


And if all parsers are satisfied [`run`](OptionParser::run) produces the result


<div class='bpaf-doc'>
$ app -i 10<br>
Options { argument: 10 }
</div>

</details>