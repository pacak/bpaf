<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}

fn parse_decision() -> impl Parser<Decision> {
    long("decision")
        .help("Positive decision")
        .flag(Decision::Yes, Decision::No)
}

pub fn options() -> OptionParser<Options> {
    let decision = parse_decision();
    construct!(Options { decision }).to_options()
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
    /// Positive decision
    #[bpaf(flag(Decision::Yes, Decision::No))]
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In `--help` output `bpaf` shows flags with no meta variable attached


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--decision</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --decision</b></tt></dt>
<dd>Positive decision</dd>
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


Presense of a long name is decoded into `Yes`

<div class='bpaf-doc'>
$ app --decision<br>
Options { decision: Yes }
</div>


Absense is `No`

<div class='bpaf-doc'>
$ app <br>
Options { decision: No }
</div>

</details>