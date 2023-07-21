<details><summary>Combinatoric example</summary>

```no_run
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Options {
    argument: BTreeSet<u32>,
    switches: BTreeSet<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .collect();
    let switches = long("switch").help("some switch").switch().collect();
    construct!(Options { argument, switches }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
use std::collections::BTreeSet;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(argument::<u32>("ARG"), collect)]
    argument: BTreeSet<u32>,
    /// some switch
    #[bpaf(long("switch"), switch, collect)]
    switches: BTreeSet<bool>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In usage lines `collect` items are indicated with `...`

<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>... [<tt><b>--switch</b></tt>]...</p><p><div>
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


Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything


<div class='bpaf-doc'>
$ app --argument 10 --argument 20 --argument 20<br>
Options { argument: {10, 20}, switches: {false} }
</div>


If there's no matching parameters - it would produce an empty set. Note, in case of
[`switch`](NamedArg::switch) parser or other parsers that can succeed without consuming anything
it would capture that value so `many` captures the first one of those.
You can use [`req_flag`](NamedArg::req_flag) to avoid that.


<div class='bpaf-doc'>
$ app <br>
Options { argument: {}, switches: {false} }
</div>


For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they produce something


<div class='bpaf-doc'>
$ app --switch --switch<br>
Options { argument: {}, switches: {true} }
</div>

</details>