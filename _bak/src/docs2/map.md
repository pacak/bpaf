<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}
pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").map(|x| x * 2);
    construct!(Options { number }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
fn twice_the_num(n: u32) -> u32 {
    n * 2
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<u32>("N"), map(twice_the_num))]
    number: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`map` don't make any changes to generated `--help` message


You can use `map` to apply arbitrary pure transformation to any input.
Here `--number` takes a numerical value and doubles it


<div class='bpaf-doc'>
$ app --number 10<br>
Options { number: 20 }
</div>


But if function inside the parser fails - user will get the error back unless it's handled
in some way. In fact here execution never reaches `map` function -
[`argument`](NamedArg::argument) tries to parse `ten` as a number, fails and reports the error


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