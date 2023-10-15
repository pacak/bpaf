## Derive example

```rust,id:1
# use bpaf::*;
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
```

## Combinatoric example

```rust,id:2
# use bpaf::*;
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
```


Fields can have different types, including `Option` or `Vec`, in this example they are two
`usize` and one `f64`.

```run,id:1,id:2
--help
```

flag `--point` takes 3 positional arguments: two integers for X and Y coordinates and one
floating point for height, order is important, switch `--rotate` can go on either side of it

```run,id:1,id:2
--rotate --point 10 20 3.1415
```

parser accepts multiple points, they must not interleave

```run,id:1,id:2
--point 10 20 3.1415 --point 1 2 0.0
```

And `--rotate` can't go in the middle of the point definition

```run,id:1,id:2
--point 10 20 --rotate 3.1415
```
