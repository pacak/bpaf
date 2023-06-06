<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone)]
pub struct MultiArg {
    set: (),
    name: String,
    value: String,
}

pub fn options() -> OptionParser<Options> {
    let set = long("set").req_flag(());
    let name = positional("NAME").help("Name for the option");
    let value = positional("VAL").help("Value to set");
    let multi_arg = construct!(MultiArg { set, name, value })
        .adjacent()
        .optional();

    let turbo = long("turbo").switch();
    construct!(Options { multi_arg, turbo }).to_options()
}
```

</details>
<details><summary>Derive example</summary>

```no_run
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, optional)]
    multi_arg: Option<MultiArg>,
    turbo: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
pub struct MultiArg {
    #[bpaf(long)]
    set: (),
    #[bpaf(positional("NAME"))]
    /// Name for the option
    name: String,
    #[bpaf(positional("VAL"))]
    /// Value to set
    value: String,
}
```

</details>
<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--set</b></tt> <tt><i>NAME</i></tt> <tt><i>VAL</i></tt>] [<tt><b>--turbo</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><tt><b>--set</b></tt> <tt><i>NAME</i></tt> <tt><i>VAL</i></tt><dt><tt><i>NAME</i></tt></dt>
<dd>Name for the option</dd>
<dt><tt><i>VAL</i></tt></dt>
<dd>Value to set</dd>
<p></p><dt><tt><b>    --turbo</b></tt></dt>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


? It's possible to implement multi argument options by using required flag followed by one or
? more positional items

<div class='bpaf-doc'>
$ app --turbo --set name Bob<br>
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }
</div>

OK
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }

? Other flags can go on either side of items

<div class='bpaf-doc'>
$ app --set name Bob --turbo<br>
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }
</div>

OK
Options { multi_arg: Some(MultiArg { set: (), name: "name", value: "Bob" }), turbo: true }

? But not in between

<div class='bpaf-doc'>
$ app --set name --turbo Bob<br>
Expected <tt><i>VAL</i></tt>, got <b>--turbo</b>. Pass <tt><b>--help</b></tt> for usage information
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>

Stderr
Expected <VAL>, got "--turbo". Pass --help for usage information
</details>