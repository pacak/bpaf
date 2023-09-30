
```no_run
#[derive(Debug, Clone)]
pub struct Options {
    package: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let help = long("help").short('H').help("Renders help information");
    let version = long("version")
        .short('v')
        .help("Renders version information");
    let package = short('p')
        .help("Package to check")
        .argument("SPEC")
        .optional();

    construct!(Options { package })
        .to_options()
        .descr("Command with custom flags for help and version")
        .version("0.42")
        .help_parser(help)
        .version_parser(version)
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

This example replaces description and short name for `--help` parser. Long name works as is


<div class='bpaf-doc'>
$ app --help<br>
<p>Command with custom flags for help and version</p><p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-p</b></tt>=<tt><i>SPEC</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-p</b></tt>=<tt><i>SPEC</i></tt></dt>
<dd>Package to check</dd>
<dt><tt><b>-H</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Renders help information</dd>
<dt><tt><b>-v</b></tt>, <tt><b>--version</b></tt></dt>
<dd>Renders version information</dd>
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


Short name is now capitalized


<div class='bpaf-doc'>
$ app -H<br>
<p>Command with custom flags for help and version</p><p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-p</b></tt>=<tt><i>SPEC</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-p</b></tt>=<tt><i>SPEC</i></tt></dt>
<dd>Package to check</dd>
<dt><tt><b>-H</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Renders help information</dd>
<dt><tt><b>-v</b></tt>, <tt><b>--version</b></tt></dt>
<dd>Renders version information</dd>
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


and old short name no longer works.


<div class='bpaf-doc'>
$ app -h<br>
<b>Error:</b> <b>-h</b> is not expected in this context
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


Same with `--version` parser - new description, original long name and custom short name are
both working


<div class='bpaf-doc'>
$ app --version<br>
<p>Version: 0.42</p>
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
$ app -v<br>
<p>Version: 0.42</p>
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