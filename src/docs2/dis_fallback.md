<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    jobs: usize,
}

pub fn options() -> OptionParser<Options> {
    let jobs = long("jobs")
        .help("Number of jobs")
        .argument("JOBS")
        .fallback(42)
        .display_fallback();
    construct!(Options { jobs }).to_options()
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
#[allow(dead_code)]
pub struct Options {
    /// Number of jobs
    #[bpaf(argument("JOBS"), fallback(42), display_fallback)]
    jobs: usize,
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
Options { jobs: 42 }
</div>


If value is present - fallback value is ignored


<div class='bpaf-doc'>
$ app --jobs 10<br>
Options { jobs: 10 }
</div>


Parsing errors are preserved and presented to the user


<div class='bpaf-doc'>
$ app --jobs ten<br>
<b>Error:</b> couldn't parse <b>ten</b>: invalid digit found in string
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
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--jobs</b></tt>=<tt><i>JOBS</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --jobs</b></tt>=<tt><i>JOBS</i></tt></dt>
<dd>Number of jobs</dd>
<dt></dt>
<dd>[default: 42]</dd>
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