<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number")
        .argument::<String>("N")
        // normally you'd use argument::<u32> to get a numeric
        // value and `map` to double it
        .parse::<_, _, ParseIntError>(|s| Ok(u32::from_str(&s)? * 2));
    construct!(Options { number }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
fn twice_the_num(s: String) -> Result<u32, ParseIntError> {
    Ok(u32::from_str(&s)? * 2)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<String>("N"), parse(twice_the_num))]
    number: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`parse` don't make any changes to generated `--help` message


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


You can use `parse` to apply arbitrary failing transformation to any input.
For example here `--number` takes a numerical value and doubles it


<div class='bpaf-doc'>
$ app --number 10<br>
Options { number: 20 }
</div>


But if function inside the parser fails - user will get the error back unless it's handled
in some other way


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