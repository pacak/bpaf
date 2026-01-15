<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    height: Vec<usize>,
    height_str: Vec<String>,
    width: Vec<usize>,
    width_str: Vec<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .some("You must specify some heights")
        .catch();

    let height_str = long("height").argument::<String>("PX").many().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .some("You must specify some widths");

    let width_str = long("width").argument::<String>("PX").many().hide();

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
    #[bpaf(long, argument("PX"), some("You must specify some heights"), catch)]
    /// Height of a rectangle
    height: Vec<usize>,

    #[bpaf(long("height"), argument("PX"), many, hide)]
    height_str: Vec<String>,

    #[bpaf(long, argument("PX"), some("You must specify some widths"))]
    /// Width of a rectangle
    width: Vec<usize>,

    #[bpaf(long("width"), argument("PX"), many, hide)]
    width_str: Vec<String>,
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
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--height</b></tt>=<tt><i>PX</i></tt>... <tt><b>--width</b></tt>=<tt><i>PX</i></tt>...</p><p><div>
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


When executed with no parameters parser fails because `some` requires you to specify at least
one matching parameter


<div class='bpaf-doc'>
$ app <br>
<b>Error:</b> You must specify some heights
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


When executed with expected parameters fields with `usize` get their values


<div class='bpaf-doc'>
$ app --height 100 --width 100 --height 12 --width 44<br>
Options { height: [100, 12], height_str: [], width: [100, 44], width_str: [] }
</div>


With incorrect value for `--height` parameter inner part of `height` parser fails, `some`
combined with `catch` handles this failure and produces `[]` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string


<div class='bpaf-doc'>
$ app --height 10 --height twenty --width 33<br>
Options { height: [10], height_str: ["twenty"], width: [33], width_str: [] }
</div>


In case of wrong `--width` - parser `width` fails, parser for `some` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser


<div class='bpaf-doc'>
$ app --height 10 --width 33 --width ten<br>
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