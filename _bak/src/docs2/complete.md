<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
}

fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    names
        .iter()
        .filter(|name| name.starts_with(input))
        .map(|name| (*name, None))
        .collect::<Vec<_>>()
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify character's name")
        .argument("NAME")
        .complete(completer);
    construct!(Options { name }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
/// suggest completions for the input
fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    names
        .iter()
        .filter(|name| name.starts_with(input))
        .map(|name| (*name, None))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("NAME"), complete(completer))]
    /// Specify character's name
    name: String,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`complete` annotation does not affect parsing results or generated help message


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-n</b></tt>=<tt><i>NAME</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-n</b></tt>, <tt><b>--name</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Specify character's name</dd>
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



<div class='bpaf-doc'>
$ app --name Bob<br>
Options { name: "Bob" }
</div>


But when invoked with shell completion can generate suggestions for user to what to type:

```console
$ app --name L<TAB>
$ app --name Lupisregina
```
</details>