<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    name: String,
    output: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help(
            "\
Output detailed help information, you can specify it multiple times

 when used once it outputs basic diagnostic info,
 when used twice or three times - it includes extra debugging.",
            // ^ note extra spaces before "when" that preserve the linebreaks
        )
        .switch();
    let name = long("name")
        .help("Use this as a task name")
        .argument("NAME");

    let output = positional("OUTPUT")
        .help("Save output to a file")
        .optional();

    construct!(Options {
        verbose,
        name,
        output
    })
    .to_options()
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
    /// Output detailed help information, you can specify it multiple times
    ///
    ///  when used once it outputs basic diagnostic info,
    ///  when used twice or three times - it includes extra debugging.
    //  ^ note extra spaces before when that preserve the linebreaks
    verbose: bool,

    #[bpaf(argument("NAME"))]
    /// Use this as a task name
    name: String,

    #[bpaf(positional("OUTPUT"))]
    /// Save output to a file
    output: Option<String>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

When `--help` used once it renders shoter version of the help information


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>] <tt><b>--name</b></tt>=<tt><i>NAME</i></tt> [<tt><i>OUTPUT</i></tt>]</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>OUTPUT</i></tt></dt>
<dd>Save output to a file</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Output detailed help information, you can specify it multiple times</dd>
<dt><tt><b>    --name</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Use this as a task name</dd>
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


When used twice - it renders full version. Documentation generator uses full
version as well


<div class='bpaf-doc'>
$ app --help --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>] <tt><b>--name</b></tt>=<tt><i>NAME</i></tt> [<tt><i>OUTPUT</i></tt>]</p><p><div>
<b>Available positional items:</b></div><dl><dt><tt><i>OUTPUT</i></tt></dt>
<dd>Save output to a file</dd>
</dl>
</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Output detailed help information, you can specify it multiple times<br>
 when used once it outputs basic diagnostic info,<br>
when used twice or three times - it includes extra debugging.</dd>
<dt><tt><b>    --name</b></tt>=<tt><i>NAME</i></tt></dt>
<dd>Use this as a task name</dd>
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


Presence or absense of a help message should not affect the parser's output


<div class='bpaf-doc'>
$ app --name Bob output.txt<br>
Options { verbose: false, name: "Bob", output: Some("output.txt") }
</div>

</details>