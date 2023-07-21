<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number")
        .help(
            &[
                ("Very", Style::Emphasis),
                (" important argument", Style::Text),
            ][..],
        )
        .argument::<u32>("N");
    construct!(Options { number }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run

const ARG: &[(&str, Style)] = &[
    ("Very", Style::Emphasis),
    (" important argument", Style::Text),
];

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("N"), help(ARG))]
    number: u32,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--number</b></tt>=<tt><i>N</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --number</b></tt>=<tt><i>N</i></tt></dt>
<dd><b>Very</b> important argument</dd>
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

</details>