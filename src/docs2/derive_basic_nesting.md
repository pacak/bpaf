
```no_run
#[derive(Debug, Clone, Bpaf)]
pub enum Format {
    /// Produce output in HTML format
    Html,
    /// Produce output in Markdown format
    Markdown,
    /// Produce output in manpage format
    Manpage,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// File to process
    input: String,
    #[bpaf(external(format))]
    format: Format,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help message lists all possible options


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--input</b></tt>=<tt><i>ARG</i></tt> (<tt><b>--html</b></tt> | <tt><b>--markdown</b></tt> | <tt><b>--manpage</b></tt>)</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --input</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>File to process</dd>
<dt><tt><b>    --html</b></tt></dt>
<dd>Produce output in HTML format</dd>
<dt><tt><b>    --markdown</b></tt></dt>
<dd>Produce output in Markdown format</dd>
<dt><tt><b>    --manpage</b></tt></dt>
<dd>Produce output in manpage format</dd>
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


Parser accepts one and only one value from enum in this example


<div class='bpaf-doc'>
$ app --input Cargo.toml --html<br>
Options { input: "Cargo.toml", format: Html }
</div>


<div class='bpaf-doc'>
$ app --input Cargo.toml --manpage<br>
Options { input: "Cargo.toml", format: Manpage }
</div>



<div class='bpaf-doc'>
$ app --input hello<br>
<b>Error:</b> expected <tt><b>--html</b></tt>, <tt><b>--markdown</b></tt>, or more, pass <tt><b>--help</b></tt> for usage information
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