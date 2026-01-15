
```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, switch)]
    switch: bool,

    /// Custom number
    #[bpaf(positional("NUM"))]
    argument: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

`bpaf` generates help message with a short name only as described


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-s</b></tt>] <tt><i>NUM</i></tt></p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>NUM</i></tt></dt>
<dd>Custom number</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt></dt>
<dd>A custom switch</dd>
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


And accepts the short name only


<div class='bpaf-doc'>
$ app -s 42<br>
Options { switch: true, argument: 42 }
</div>


long name is missing


<div class='bpaf-doc'>
$ app --switch 42<br>
<b>Error:</b> <b>--switch</b> is not expected in this context
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