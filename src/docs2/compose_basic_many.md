
```no_run
use bpaf::*;

pub fn options() -> OptionParser<Vec<u32>> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .many();
    argument.to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything

<div class='bpaf-doc'>
$ app --argument 10 --argument 20<br>
[10, 20]
</div>


If there's no matching parameters - it would produce an empty vector.

<div class='bpaf-doc'>
$ app <br>
[]
</div>


In usage lines `many` items are indicated with `...`

<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>important argument</dd>
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