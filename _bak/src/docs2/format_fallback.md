<details><summary>Combinatoric example</summary>

```no_run
use std::{fmt::Display as _, path::PathBuf};
#[derive(Debug, Clone)]
pub struct Options {
    log_file: PathBuf,
}

pub fn options() -> OptionParser<Options> {
    let log_file = long("log-file")
        .help("Path to log file")
        .argument::<PathBuf>("FILE")
        .guard(
            |log_file| !log_file.is_dir(),
            "The log file can't be a directory",
        )
        .fallback(PathBuf::from("logfile.txt"))
        .format_fallback(|path, f| path.display().fmt(f));
    construct!(Options { log_file }).to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Derive example</summary>

```no_run
use std::{fmt::Display as _, path::PathBuf};
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
pub struct Options {
    /// Path to log file
    #[bpaf(
        argument("FILE"),
        guard(|log_file| !log_file.is_dir(), "The log file can't be a directory"),
        fallback(PathBuf::from("logfile.txt")),
        format_fallback(|path, f| path.display().fmt(f)),
    )]
    log_file: PathBuf,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

`fallback` changes parser to fallback to a default value used when argument is not specified


<div class='bpaf-doc'>
$ app <br>
Options { log_file: "logfile.txt" }
</div>


If value is present - fallback value is ignored


<div class='bpaf-doc'>
$ app --log-file output.txt<br>
Options { log_file: "output.txt" }
</div>


Parsing errors are preserved and presented to the user


<div class='bpaf-doc'>
$ app --log-file /<br>
<b>Error:</b> <b>/</b>: The log file can't be a directory
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


With [`display_fallback`](ParseFallback::display_fallback),
[`debug_fallback`](ParseFallback::debug_fallback), and
[`format_fallback`](ParseFallback::format_fallback), you can make it so the default value
is visible in the `--help` output.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--log-file</b></tt>=<tt><i>FILE</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --log-file</b></tt>=<tt><i>FILE</i></tt></dt>
<dd>Path to log file</dd>
<dt></dt>
<dd>[default: logfile.txt]</dd>
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