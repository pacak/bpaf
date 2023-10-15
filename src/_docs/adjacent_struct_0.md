## Derive example

````rust
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
````

## Combinatoric example

````rust
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
````

Fields can have different types, including `Option` or `Vec`, in this example they are two
`usize` and one `f64`.



```text
$ app --help
Usage: app [-p X Y Z]... [-r]

Available options:
  -p X Y Z
    -p, --point   Point coordinates
    X             X coordinate of a point
    Y             Y coordinate of a point
    Z             Height of a point above the plane

    -r, --rotate  Face the camera towards the first point
    -h, --help    Prints help information
```


flag `--point` takes 3 positional arguments: two integers for X and Y coordinates and one
floating point for height, order is important, switch `--rotate` can go on either side of it



```text
$ app --rotate --point 10 20 3.1415
Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }], rotate: true }
```


parser accepts multiple points, they must not interleave



```text
$ app --point 10 20 3.1415 --point 1 2 0.0
Options { point: [Point { point: (), x: 10, y: 20, z: 3.1415 }, Point { point: (), x: 1, y: 2, z: 0.0 }], rotate: false }
```


And `--rotate` can't go in the middle of the point definition



```text
$ app --point 10 20 --rotate 3.1415
Error: expected `Z`, pass `--help` for usage information
```

