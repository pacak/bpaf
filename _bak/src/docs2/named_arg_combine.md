
```no_run
#[derive(Debug, Clone)]
pub enum Output {
    ToFile(PathBuf),
    ToConsole,
}
pub fn options() -> OptionParser<(usize, Output, bool)> {
    // In most cases you don't keep `NamedArg` around long enough
    // to assign it a name
    let size = short('s')
        .long("size")
        .help("Maximum size to process")
        .argument("SIZE");

    // but it can be useful if you want to have several arguments
    // sharing exact set of names - for example a switch (req_flag)
    // and an argument;
    let output = short('o').long("output");

    let to_file = output
        .clone()
        .help("Save output to file")
        .argument("PATH")
        .map(Output::ToFile);
    let to_console = output
        .help("Print output to console")
        .req_flag(Output::ToConsole);

    // when combining multiple parsers that can conflict with each other
    // it's a good idea to put more general first:
    let output = construct!([to_file, to_console]);

    let verbose = short('v')
        .long("verbose")
        .long("detailed")
        .help("Produce a detailed report")
        .switch();

    construct!(size, output, verbose).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

`--help` output will contain first short and first long names that are present and won't have
anything about hidden aliases.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>-s</b></tt>=<tt><i>SIZE</i></tt> (<tt><b>-o</b></tt>=<tt><i>PATH</i></tt> | <tt><b>-o</b></tt>) [<tt><b>-v</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-s</b></tt>, <tt><b>--size</b></tt>=<tt><i>SIZE</i></tt></dt>
<dd>Maximum size to process</dd>
<dt><tt><b>-o</b></tt>, <tt><b>--output</b></tt>=<tt><i>PATH</i></tt></dt>
<dd>Save output to file</dd>
<dt><tt><b>-o</b></tt>, <tt><b>--output</b></tt></dt>
<dd>Print output to console</dd>
<dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Produce a detailed report</dd>
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


`--detailed` is a hidden alias and still works despite not being present in `--help` output
above


<div class='bpaf-doc'>
$ app -o -s 2 --detailed<br>
(2, ToConsole, true)
</div>


And hidden means actually hidden. While error message can suggest to fix a typo to make it a
valid _visible_ argument


<div class='bpaf-doc'>
$ app -o best.txt -s 10 --verbos<br>
<b>Error:</b> no such flag: <b>--verbos</b>, did you mean <tt><b>--verbose</b></tt>?
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


It will not do so for hidden aliases


<div class='bpaf-doc'>
$ app -o best.txt -s 10 --detaile<br>
<b>Error:</b> <b>--detaile</b> is not expected in this context
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



In this example names `-o` and `--output` can be parsed by two parsers - `to_file` and
`to_console`, first one succeeds only if `-o` is followed by a non option name, `best.txt`.


<div class='bpaf-doc'>
$ app -o best.txt --size 10<br>
(10, ToFile("best.txt"), false)
</div>


If such name is not present - parser will try to consume one without, producing `ToConsole`
variant.


<div class='bpaf-doc'>
$ app -o -s 42<br>
(42, ToConsole, false)
</div>


If neither is present - it fails - parser for `output` expects one of its branches to succeed


<div class='bpaf-doc'>
$ app -s 330<br>
<b>Error:</b> expected <tt><b>--output</b></tt>=<tt><i>PATH</i></tt> or <tt><b>--output</b></tt>, pass <tt><b>--help</b></tt> for usage information
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


But this can be fixed with [`optional`](Parser::optional) (not included in this example).
</details>