<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    money: u32,
}

pub fn options() -> OptionParser<Options> {
    // User can customise a name
    let name = long("name").help("Use a custom user name").argument("NAME");
    // but not starting amount of money
    let money = pure(330);
    construct!(Options { name, money }).to_options()
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
    #[bpaf(argument("NAME"))]
    /// Use a custom user name
    name: String,
    #[bpaf(pure(330))]
    money: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`pure` does not show up in `--help` message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--name</b></tt>=<tt><i>NAME</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --name</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Use a custom user name</dd>
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


And there's no way to alter the value from the command line


<div class='bpaf-doc'>
$ app --name Bob<br>
Options { name: "Bob", money: 330 }
</div>


Any attempts to do so would result in an error :)


<div class='bpaf-doc'>
$ app --money 100000 --name Hackerman<br>
<b>Error:</b> <b>--money</b> is not expected in this context
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