<details><summary>Combinatoric example</summary>

```no_run
#[derive(Debug, Clone)]
pub struct Options {
    point: Vec<Point>,
    rotate: bool,
}

#[derive(Debug, Clone)]
struct Point {
    point: (),
    x: usize,
    y: usize,
    z: f64,
}

fn point() -> impl Parser<Point> {
    let point = short('p')
        .long("point")
        .help("Point coordinates")
        .req_flag(());
    let x = positional::<usize>("X").help("X coordinate of a point");
    let y = positional::<usize>("Y").help("Y coordinate of a point");
    let z = positional::<f64>("Z").help("Height of a point above the plane");
    construct!(Point { point, x, y, z }).adjacent()
}

pub fn options() -> OptionParser<Options> {
    let rotate = short('r')
        .long("rotate")
        .help("Face the camera towards the first point")
        .switch();
    let point = point().many();
    construct!(Options { point, rotate }).to_options()
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
    point: Vec<Point>,
    #[bpaf(short, long)]
    /// Face the camera towards the first point
    rotate: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Point {
    #[bpaf(short, long)]
    /// Point coordinates
    point: (),
    #[bpaf(positional("X"))]
    /// X coordinate of a point
    x: usize,
    #[bpaf(positional("Y"))]
    /// Y coordinate of a point
    y: usize,
    #[bpaf(positional("Z"))]
    /// Height of a point above the plane
    z: f64,
}

fn main() {
    println!("{:?}", options().run())
}
```

</details>
<details><summary>Output</summary>

Fields can have different types, including `Option` or `Vec`, in this example they are two
`usize` and one `f64`.


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-p</b></tt> <tt><i>X</i></tt> <tt><i>Y</i></tt> <tt><i>Z</i></tt>]... [<tt><b>-r</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><div style='padding-left: 0.5em'><tt><b>-p</b></tt> <tt><i>X</i></tt> <tt><i>Y</i></tt> <tt><i>Z</i></tt></div><dt><tt><b>-p</b></tt>, <tt><b>--point</b></tt></dt>
<dd>Point coordinates</dd>
<dt><tt><i>X</i></tt></dt>
<dd>X coordinate of a point</dd>
<dt><tt><i>Y</i></tt></dt>
<dd>Y coordinate of a point</dd>
<dt><tt><i>Z</i></tt></dt>
<dd>Height of a point above the plane</dd>
<p></p><dt><tt><b>-r</b></tt>, <tt><b>--rotate</b></tt></dt>
<dd>Face the camera towards the first point</dd>
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


flag `--point` takes 3 positional arguments: two integers for X and Y coordinates and one floating point for height, order is
important, switch `--rotate` can go on either side of it


<div class='bpaf-doc'>
$ app --rotate --point 10 20 3.1415<br>
Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }], rotate: true }
</div>


parser accepts multiple points, they must not interleave


<div class='bpaf-doc'>
$ app --point 10 20 3.1415 --point 1 2 0.0<br>
Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }, Point { point: (), x: 1, y: 2, z: 0.0 }], rotate: false }
</div>


`--rotate` can't go in the middle of the point definition as the parser expects the second item


<div class='bpaf-doc'>
$ app --point 10 20 --rotate 3.1415<br>
<b>Error:</b> expected <tt><i>Z</i></tt>, pass <tt><b>--help</b></tt> for usage information
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