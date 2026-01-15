<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    argument: Vec<u32>,
    switches: Vec<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .some("want at least one argument");
    let switches = long("switch")
        .help("some switch")
        .req_flag(true)
        .some("want at least one switch");
    construct!(Options { argument, switches }).to_options()
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
    #[bpaf(argument("ARG"), some("want at least one argument"))]
    argument: Vec<u32>,
    /// some switch
    #[bpaf(long("switch"), req_flag(true), some("want at least one switch"))]
    switches: Vec<bool>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In usage lines `some` items are indicated with `...`


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--argument</b></tt>=<tt><i>ARG</i></tt>... <tt><b>--switch</b></tt>...</p><p><div>
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


Run inner parser as many times as possible collecting all the new results, but unlike
`many` needs to collect at least one element to succeed


<div class='bpaf-doc'>
$ app --argument 10 --argument 20 --switch<br>
Options { argument: [10, 20], switches: [true] }
</div>


With not enough parameters to satisfy both parsers at least once - it fails


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> want at least one argument
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


both parsers need to succeed to create a struct


<div class='bpaf-doc'>
$ app --argument 10<br>
<b>Error:</b> want at least one switch
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


 For parsers that can succeed without consuming anything such as `flag` or `switch` - `some`
only collects values as long as they produce something


<div class='bpaf-doc'>
$ app --switch --argument 10<br>
Options { argument: [10], switches: [true] }
</div>

</details>