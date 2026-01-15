
```no_run
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Specify user name
    name: String,

    /// Specify user age
    age: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--name</b></tt>=<tt><i>ARG</i></tt> <tt><b>--age</b></tt>=<tt><i>ARG</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --name</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Specify user name</dd>
<dt><tt><b>    --age</b></tt>=<tt><i>ARG</i></tt></dt>
<dd>Specify user age</dd>
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


`--help` shows arguments as a short name with attached metavariable

Value can be separated from flag by space, `=` sign


<div class='bpaf-doc'>
$ app --name Bob --age 12<br>
Options { name: "Bob", age: 12 }
</div>


<div class='bpaf-doc'>
$ app --name "Bob" --age=12<br>
Options { name: "Bob", age: 12 }
</div>


<div class='bpaf-doc'>
$ app --name=Bob<br>
<b>Error:</b> expected <tt><b>--age</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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
$ app --name="Bob"<br>
<b>Error:</b> expected <tt><b>--age</b></tt>=<tt><i>ARG</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


Or in case of short name - be directly adjacent to it


<div class='bpaf-doc'>
$ app -nBob<br>
<b>Error:</b> expected <tt><b>--name</b></tt>=<tt><i>ARG</i></tt>, got <b>-nBob</b>. Pass <tt><b>--help</b></tt> for usage information
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


For long names - this doesn't work since parser can't tell where name
stops and argument begins:


<div class='bpaf-doc'>
$ app --age12<br>
<b>Error:</b> no such flag: <b>--age12</b>, did you mean <tt><b>--age</b></tt>?
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


Either way - value is required, passing just the argument name results in parse failure


<div class='bpaf-doc'>
$ app --name<br>
<b>Error:</b> <tt><b>--name</b></tt> requires an argument <tt><i>ARG</i></tt>
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