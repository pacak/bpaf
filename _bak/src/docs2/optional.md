<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    version: Option<usize>,
    feature: Option<String>,
}
pub fn options() -> OptionParser<Options> {
    let version = long("version").argument("VERS").optional();
    let feature = long("feature").argument("FEAT").optional();
    construct!(Options { version, feature }).to_options()
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
    #[bpaf(argument("VERS"))]
    version: Option<usize>,
    #[bpaf(argument("FEAT"))]
    feature: Option<String>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`bpaf` encases optional arguments in usage with `[]`


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--version</b></tt>=<tt><i>VERS</i></tt>] [<tt><b>--feature</b></tt>=<tt><i>FEAT</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --version</b></tt>=<tt><i>VERS</i></tt></dt>
<dt><tt><b>    --feature</b></tt>=<tt><i>FEAT</i></tt></dt>
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


Missing arguments are turned into None


<div class='bpaf-doc'>
$ app <br>
Options { version: None, feature: None }
</div>


Present values are `Some`


<div class='bpaf-doc'>
$ app --version 10<br>
Options { version: Some(10), feature: None }
</div>


As usual you can specify both


<div class='bpaf-doc'>
$ app --version 10 --feature feat<br>
Options { version: Some(10), feature: Some("feat") }
</div>

</details>