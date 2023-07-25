
```no_run
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    switch: bool,

    /// A custom argument
    argument: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

bpaf generates a help message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--switch</b></tt>] <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --switch</b></tt></dt>
<dd>A custom switch</dd>
<dt><tt><b>    --argument</b></tt>=<tt><i>ARG</i></tt></dt>
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


And two parsers. Numeric argument is required, boolean switch is optional and fall back value
is false.


<div class='bpaf-doc'>
$ app --switch<br>
<b>Error:</b> expected <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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
$ app --switch --argument 42<br>
Options { switch: true, argument: 42 }
</div>



<div class='bpaf-doc'>
$ app --argument 42<br>
Options { switch: false, argument: 42 }
</div>

</details>