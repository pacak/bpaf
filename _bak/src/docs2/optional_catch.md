<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    height: Option<usize>,
    height_str: Option<String>,
    width: Option<usize>,
    width_str: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .optional()
        .catch();

    let height_str = long("height").argument::<String>("PX").optional().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .optional();

    let width_str = long("width").argument::<String>("PX").optional().hide();

    construct!(Options {
        height,
        height_str,
        width,
        width_str
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
    #[bpaf(long, argument("PX"), optional, catch)]
    /// Height of a rectangle
    height: Option<usize>,

    #[bpaf(long("height"), argument("PX"), optional, hide)]
    height_str: Option<String>,

    #[bpaf(long, argument("PX"), optional)]
    /// Width of a rectangle
    width: Option<usize>,

    #[bpaf(long("width"), argument("PX"), optional, hide)]
    width_str: Option<String>,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Despite parser producing a funky value - help looks like you would expect from a parser that
takes two values


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--height</b></tt>=<tt><i>PX</i></tt>] [<tt><b>--width</b></tt>=<tt><i>PX</i></tt>]</p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --height</b></tt>=<tt><i>PX</i></tt></dt>
<dd>Height of a rectangle</dd>
<dt><tt><b>    --width</b></tt>=<tt><i>PX</i></tt></dt>
<dd>Width of a rectangle</dd>
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


When executed with no parameters it produces four `None` values - all parsers succeed by the
nature of them being [`optional`](Parser::optional)


<div class='bpaf-doc'>
$ app <br>
Options { height: None, height_str: None, width: None, width_str: None }
</div>


When executed with expected parameters fields with `usize` get their values


<div class='bpaf-doc'>
$ app --height 100 --width 100<br>
Options { height: Some(100), height_str: None, width: Some(100), width_str: None }
</div>


With incorrect value for `--height` parameter inner part of `height` parser fails, `optional`
combined with `catch` handles this failure and produces `None` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string


<div class='bpaf-doc'>
$ app --height ten<br>
Options { height: None, height_str: Some("ten"), width: None, width_str: None }
</div>


In case of wrong `--width` - parser `width` fails, parser for `optional` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser


<div class='bpaf-doc'>
$ app --width ten<br>
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

</details>