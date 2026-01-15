<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    rect: Vec<Rect>,
    mirror: bool,
}

#[derive(Debug, Clone)]
struct Rect {
    rect: (),
    width: usize,
    height: usize,
    painted: bool,
}

fn rect() -> impl Parser<Rect> {
    let rect = long("rect").help("Define a new rectangle").req_flag(());
    let width = short('w')
        .long("width")
        .help("Rectangle width in pixels")
        .argument::<usize>("PX");
    let height = short('h')
        .long("height")
        .help("Rectangle height in pixels")
        .argument::<usize>("PX");
    let painted = short('p')
        .long("painted")
        .help("Should rectangle be filled?")
        .switch();
    construct!(Rect {
        rect,
        width,
        height,
        painted,
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let mirror = long("mirror").help("Mirror the image").switch();
    let rect = rect().many();
    construct!(Options { rect, mirror }).to_options()
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
    #[bpaf(external, many)]
    rect: Vec<Rect>,
    /// Mirror the image
    mirror: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Rect {
    /// Define a new rectangle
    rect: (),
    #[bpaf(short, long, argument("PX"))]
    /// Rectangle width in pixels
    width: usize,
    #[bpaf(short, long, argument("PX"))]
    /// Rectangle height in pixels
    height: usize,
    #[bpaf(short, long)]
    /// Should rectangle be filled?
    painted: bool,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

This example parses multipe rectangles from a command line defined by dimensions and the fact
if its filled or not, to make things more interesting - every group of coordinates must be
prefixed with `--rect`


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>--rect</b></tt> <tt><b>-w</b></tt>=<tt><i>PX</i></tt> <tt><b>-h</b></tt>=<tt><i>PX</i></tt> [<tt><b>-p</b></tt>]]... [<tt><b>--mirror</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><div style='padding-left: 0.5em'><tt><b>--rect</b></tt> <tt><b>-w</b></tt>=<tt><i>PX</i></tt> <tt><b>-h</b></tt>=<tt><i>PX</i></tt> [<tt><b>-p</b></tt>]</div><dt><tt><b>    --rect</b></tt></dt>
<dd>Define a new rectangle</dd>
<dt><tt><b>-w</b></tt>, <tt><b>--width</b></tt>=<tt><i>PX</i></tt></dt>
<dd>Rectangle width in pixels</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--height</b></tt>=<tt><i>PX</i></tt></dt>
<dd>Rectangle height in pixels</dd>
<dt><tt><b>-p</b></tt>, <tt><b>--painted</b></tt></dt>
<dd>Should rectangle be filled?</dd>
<p></p><dt><tt><b>    --mirror</b></tt></dt>
<dd>Mirror the image</dd>
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


Order of items within the rectangle is not significant and you can have several of them,
because fields are still regular arguments - order doesn't matter for as long as they belong
to some rectangle

<div class='bpaf-doc'>
$ app --rect --width 10 --height 10 --rect --height=10 --width=10<br>
Options { rect: [Rect { rect: (), width: 10, height: 10, painted: false }, Rect { rect: (), width: 10, height: 10, painted: false }], mirror: false }
</div>


You can have optional values that belong to the group inside and outer flags in the middle

<div class='bpaf-doc'>
$ app --rect --width 10 --painted --height 10 --mirror --rect --height 10 --width 10<br>
Options { rect: [Rect { rect: (), width: 10, height: 10, painted: true }, Rect { rect: (), width: 10, height: 10, painted: false }], mirror: true }
</div>


But with `adjacent` they cannot interleave

<div class='bpaf-doc'>
$ app --rect --rect --width 10 --painted --height 10 --height 10 --width 10<br>
<b>Error:</b> expected <tt><b>--width</b></tt>=<tt><i>PX</i></tt>, pass <tt><b>--help</b></tt> for usage information
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


Or have items that don't belong to the group inside them

<div class='bpaf-doc'>
$ app --rect --width 10 --mirror --painted --height 10 --rect --height 10 --width 10<br>
<b>Error:</b> expected <tt><b>--height</b></tt>=<tt><i>PX</i></tt>, pass <tt><b>--help</b></tt> for usage information
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