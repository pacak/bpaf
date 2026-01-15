
```no_run
use bpaf::*;

#[derive(Debug, Clone)]
pub struct Options {
    argument: Vec<u32>,
    switches: Vec<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .many();
    let switches = long("switch").help("some switch").switch().many();
    construct!(Options { argument, switches }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Help message describes all the parser combined

<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>]... [<tt><b>--switch</b></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --argument</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>important argument</dd>
<dt><tt><b>    --switch</b></tt></dt>
<dd>some switch</dd>
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


And users can pass any combinations of options, resulting parser will handle them as long as they are valid


<div class='bpaf-doc'>
$ app --argument 10 --argument 20<br>
Options { argument: [10, 20], switches: [false] }
</div>


<div class='bpaf-doc'>
$ app --switch<br>
Options { argument: [], switches: [true] }
</div>


<div class='bpaf-doc'>
$ app --switch --switch --argument 20<br>
Options { argument: [20], switches: [true, true] }
</div>

</details>