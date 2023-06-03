<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    binary: Option<String>,
    package: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let binary = short('b')
        .long("binary")
        .help("Binary to run")
        .argument("BIN")
        .optional()
        .custom_usage(
            &[
                ("--binary", Style::Literal),
                ("=", Style::Text),
                ("BINARY", Style::Metavar),
            ][..],
        );

    let package = short('p')
        .long("package")
        .help("Package to check")
        .argument("PACKAGE")
        .optional();

    construct!(Options { binary, package }).to_options()
}
```

</details>
<details><summary>Output</summary>

`custom_usage` changes how parser shows up in the "Usage" section of generated `--help`, note
lack of `[]`, long name instead of a short one and different metavariable value


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--binary</b></tt>=<tt><i>BINARY</i></tt> [<tt><b>-p</b></tt>=<tt><i>PACKAGE</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-b</b></tt>, <tt><b>--binary</b></tt>=<tt><i>BIN</i></tt></dt>
<dd>Binary to run</dd>
<dt><tt><b>-p</b></tt>, <tt><b>--package</b></tt>=<tt><i>PACKAGE</i></tt></dt>
<dd>Package to check</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


Parsing behavior stays unchanged


<div class='bpaf-doc'>
$ app --binary cargo-asm --package cargo-show-asm<br>
Options { binary: Some("cargo-asm"), package: Some("cargo-show-asm") }
</div>


</details>