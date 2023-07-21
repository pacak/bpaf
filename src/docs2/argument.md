<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    age: usize,
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    let age = long("age")
        .help("Specify user age")
        // but often rust can figure it out from the context,
        // here age is going to be `usize`
        .argument("AGE")
        .fallback(18)
        .display_fallback();

    construct!(Options { name, age }).to_options()
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
    // you can specify exact type argument should produce
    // for as long as it implements `FromStr`
    #[bpaf(short, long, argument::<String>("NAME"))]
    /// Specify user name
    name: String,
    // but often rust can figure it out from the context,
    // here age is going to be `usize`
    #[bpaf(argument("AGE"), fallback(18), display_fallback)]
    /// Specify user age
    age: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-n</b></tt>=<tt><i>NAME</i></tt> [<tt><b>--age</b></tt>=<tt><i>AGE</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-n</b></tt>, <tt><b>--name</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Specify user name</dd>
<dt><tt><b>    --age</b></tt>=<tt><i>AGE</i></tt></dt>
<dd>Specify user age</dd>
<dt></dt>
<dd>[default: 18]</dd>
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
Options { name: "Bob", age: 18 }
</div>


<div class='bpaf-doc'>
$ app --name="Bob"<br>
Options { name: "Bob", age: 18 }
</div>


Or in case of short name - be directly adjacent to it


<div class='bpaf-doc'>
$ app -nBob<br>
Options { name: "Bob", age: 18 }
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
<b>Error:</b> <tt><b>--name</b></tt> requires an argument <tt><i>NAME</i></tt>
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