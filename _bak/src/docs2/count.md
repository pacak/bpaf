<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    verbosity: usize,
}

pub fn options() -> OptionParser<Options> {
    let verbosity = short('v')
        .long("verbose")
        .help("Increase the verbosity level")
        .req_flag(())
        .count();

    construct!(Options { verbosity }).to_options()
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
    /// Increase the verbosity level
    #[bpaf(short('v'), long("verbose"), req_flag(()), count)]
    verbosity: usize,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

In `--help` message `req_flag` look similarly to [`switch`](NamedArg::switch) and
[`flag`](NamedArg::flag)


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-v</b></tt>]...</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>-v</b></tt>, <tt><b>--verbose</b></tt></dt>
<dd>Increase the verbosity level</dd>
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


Since parser uses `req_flag` it succeeds exactly 0 times if there's no parameters


<div class='bpaf-doc'>
$ app <br>
Options { verbosity: 0 }
</div>


If it was specified - `count` tracks it a discards parsed values


<div class='bpaf-doc'>
$ app -vvv<br>
Options { verbosity: 3 }
</div>


<div class='bpaf-doc'>
$ app --verbose --verbose<br>
Options { verbosity: 2 }
</div>

</details>