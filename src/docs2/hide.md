<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    argument: u32,
    switch: bool,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .fallback(30);
    let switch = long("switch").help("secret switch").switch().hide();
    construct!(Options { argument, switch }).to_options()
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
    /// important argument
    #[bpaf(fallback(30))]
    argument: u32,
    /// secret switch
    #[bpaf(hide)]
    switch: bool,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`hide`  removes the inner parser from any help or autocompletion logic


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>]</p><p><div>
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


But doesn't change the parsing behavior in any way otherwise


<div class='bpaf-doc'>
$ app --argument 32<br>
Options { argument: 32, switch: false }
</div>



<div class='bpaf-doc'>
$ app --argument 42 --switch<br>
Options { argument: 42, switch: true }
</div>

</details>