<details><summary><tt>examples/numeric_prefix.rs</tt></summary>

```no_run
/// You can parse multiple positional elements with earlier being optional as well
/// This example takes two - optional numeric prefix and a command name:
///
/// > numeric_prefix 8 work
/// Options { prefix: Some(8), command: "work" }
///
/// > numeric_prefix sleep
/// Options { prefix: None, command: "sleep" }
///
/// Generated usage reflects that:
/// Usage: numeric_prefix [PREFIX] COMMAND
use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    prefix: Option<usize>,
    command: String,
}

pub fn options() -> OptionParser<Options> {
    let prefix = positional::<usize>("PREFIX")
        .help("Optional numeric command prefix")
        .optional()
        .catch();
    let command = positional::<String>("COMMAND").help("Required command name");

    construct!(Options { prefix, command }).to_options()
}

fn main() {
    println!("{:#?}", options().run());
}

```

</details>

<details><summary>Output</summary>

If `bpaf` can parse first positional argument as number - it becomes a numeric prefix


<div class='bpaf-doc'>
$ app 10 eat<br>
Options { prefix: Some(10), command: "eat" }
</div>


Otherwise it gets ignored


<div class='bpaf-doc'>
$ app "just eat"<br>
Options { prefix: None, command: "just eat" }
</div>



If validation passes but second argument is missing - in this example there's no fallback


<div class='bpaf-doc'>
$ app 10<br>
<b>Error:</b> expected <tt><i>COMMAND</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


Help should show that the prefix is optional


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><i>PREFIX</i></tt>] <tt><i>COMMAND</i></tt></p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>PREFIX</i></tt></dt>
<dd>Optional numeric command prefix</dd>
<dt><tt><i>COMMAND</i></tt></dt>
<dd>Required command name</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
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