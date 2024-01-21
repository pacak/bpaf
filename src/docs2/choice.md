
```no_run

#[derive(Debug, Clone)]
pub struct Options {
    desert: Option<&'static str>,
}

pub fn options() -> OptionParser<Options> {
    let desert = ["apple", "banana", "orange", "grape", "strawberry"]
        .iter()
        .map(|name| {
            long(name)
                .help("Pick one of the options")
                .req_flag(*name)
                .boxed()
        });
    let desert = choice(desert).optional();
    construct!(Options { desert }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

Here [`choice`] function is used to create an option for each possible desert item


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--apple</b></tt> | <tt><b>--banana</b></tt> | <tt><b>--orange</b></tt> | <tt><b>--grape</b></tt> | <tt><b>--strawberry</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --apple</b></tt></dt>
<dd>Pick one of the options</dd>
<dt><tt><b>    --banana</b></tt></dt>
<dd>Pick one of the options</dd>
<dt><tt><b>    --orange</b></tt></dt>
<dd>Pick one of the options</dd>
<dt><tt><b>    --grape</b></tt></dt>
<dd>Pick one of the options</dd>
<dt><tt><b>    --strawberry</b></tt></dt>
<dd>Pick one of the options</dd>
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


User can pick any item


<div class='bpaf-doc'>
$ app --apple<br>
Options { desert: Some("apple") }
</div>


Since parser consumes only one value you can't specify multiple flags of the same type


<div class='bpaf-doc'>
$ app --orange --grape<br>
<b>Error:</b> <tt><b>--grape</b></tt> cannot be used at the same time as <tt><b>--orange</b></tt>
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


And [`Parser::optional`] makes it so when value is not specified - `None` is produced instead


<div class='bpaf-doc'>
$ app <br>
Options { desert: None }
</div>

</details>