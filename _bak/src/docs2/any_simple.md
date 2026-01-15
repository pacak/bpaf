<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    turbo: bool,
    rest: Vec<OsString>,
}

pub fn options() -> OptionParser<Options> {
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let rest = any::<OsString, _, _>("REST", |x| (x != "--help").then_some(x))
        .help("app will pass anything unused to a child process")
        .many();
    construct!(Options { turbo, rest }).to_options()
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
    #[bpaf(short, long)]
    /// Engage the turbo mode
    turbo: bool,
    #[bpaf(any("REST", not_help), many)]
    /// app will pass anything unused to a child process
    rest: Vec<OsString>,
}

fn not_help(s: OsString) -> Option<OsString> {
    if s == "--help" {
        None
    } else {
        Some(s)
    }
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`--help` keeps working for as long as `any` captures only intended values - that is it ignores
`--help` flag specifically


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-t</b></tt>] [<tt><i>REST</i></tt>]...</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>REST</i></tt></dt>
<dd>app will pass anything unused to a child process</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-t</b></tt>, <tt><b>--turbo</b></tt></dt>
<dd>Engage the turbo mode</dd>
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


You can mix `any` with regular options, here [`switch`](NamedArg::switch) `turbo` works because it goes
before `rest` in the parser declaration


<div class='bpaf-doc'>
$ app --turbo git commit -m "hello world"<br>
Options { turbo: true, rest: ["git", "commit", "-m", "hello world"] }
</div>


"before" in the previous line means in the parser definition, not on the user input, here
`--turbo` gets consumed by `turbo` parser even the argument goes


<div class='bpaf-doc'>
$ app git commit -m="hello world" --turbo<br>
Options { turbo: true, rest: ["git", "commit", "-m=hello world"] }
</div>





<div class='bpaf-doc'>
$ app -- git commit -m="hello world" --turbo<br>
Options { turbo: false, rest: ["git", "commit", "-m=hello world", "--turbo"] }
</div>


<div class='bpaf-doc'>
$ app git commit -m="hello world" -- --turbo<br>
Options { turbo: false, rest: ["git", "commit", "-m=hello world", "--turbo"] }
</div>

</details>