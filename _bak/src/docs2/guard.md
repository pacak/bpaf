<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").guard(
        |n| *n <= 10,
        "Values greater than 10 are only available in the DLC pack!",
    );
    construct!(Options { number }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
fn dlc_check(number: &u32) -> bool {
    *number <= 10
}

const DLC_NEEDED: &str = "Values greater than 10 are only available in the DLC pack!";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("N"), guard(dlc_check, DLC_NEEDED))]
    number: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`guard` don't make any changes to generated `--help` message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--number</b></tt>=<tt><i>N</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --number</b></tt>=<tt><i>N</i></tt></dt>
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


You can use guard to set boundary limits or perform other checks on parsed values.
Parser accepts numbers below 10


<div class='bpaf-doc'>
$ app --number 5<br>
Options { number: 5 }
</div>


And fails with the error message on higher values:


<div class='bpaf-doc'>
$ app --number 11<br>
<b>Error:</b> <b>11</b>: Values greater than 10 are only available in the DLC pack!
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



But if function inside the parser fails - user will get the error back unless it's handled
in some way


<div class='bpaf-doc'>
$ app --number ten<br>
<b>Error:</b> couldn't parse <b>ten</b>: invalid digit found in string
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