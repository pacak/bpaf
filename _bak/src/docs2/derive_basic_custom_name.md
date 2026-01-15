
```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, long)]
    switch: bool,

    /// A custom argument
    #[bpaf(long("my-argument"), short('A'))]
    argument: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

`bpaf` uses custom names in help message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-s</b></tt>] <tt><b>-A</b></tt>=<tt><i>ARG</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt>, <tt><b>--switch</b></tt></dt>
<dd>A custom switch</dd>
<dt><tt><b>-A</b></tt>, <tt><b>--my-argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>A custom argument</dd>
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


As well as accepts them on a command line and uses in error message


<div class='bpaf-doc'>
$ app --switch<br>
<b>Error:</b> expected <tt><b>--my-argument</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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
$ app -A 42 -s<br>
Options { switch: true, argument: 42 }
</div>


</details>