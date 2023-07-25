
```no_run
fn small(size: &usize) -> bool {
    *size < 10
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // double the width
    #[bpaf(short, argument::<usize>("PX"), map(|w| w*2))]
    width: usize,

    // make sure the hight is below 10
    #[bpaf(argument::<usize>("LENGTH"), guard(small, "must be less than 10"))]
    height: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help as usual


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-w</b></tt>=<tt><i>PX</i></tt> <tt><b>--height</b></tt>=<tt><i>LENGTH</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-w</b></tt>=<tt><i>PX</i></tt></dt>
<dt><tt><b>    --height</b></tt>=<tt><i>LENGTH</i></tt></dt>
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


And parsed values are differnt from what user passes


<div class='bpaf-doc'>
$ app --width 10 --height 3<br>
<b>Error:</b> expected <tt><b>-w</b></tt>=<tt><i>PX</i></tt>, got <b>--width</b>. Pass <tt><b>--help</b></tt> for usage information
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


Additionally height cannot exceed 10


<div class='bpaf-doc'>
$ app --width 555 --height 42<br>
<b>Error:</b> expected <tt><b>-w</b></tt>=<tt><i>PX</i></tt>, got <b>--width</b>. Pass <tt><b>--help</b></tt> for usage information
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