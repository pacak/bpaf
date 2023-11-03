## Derive example

```rust,id:1
# use bpaf::*;
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
```

## Combinatoric example

```rust,id:2
# use bpaf::*;
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
```



This example parses multipe rectangles from a command line defined by dimensions and the fact
if its filled or not, every group of coordinates must be prefixed with `--rect`

```run,id:1,id:2
--help
```

Other than the initial `--rect` order of items within the rectangle is not significant and you
can have several of them, because fields are still regular arguments - order doesn't matter for
as long as they belong to some rectangle

```run,id:1,id:2
--rect --width 10 --height 10 --rect --height=10 --width=10
```

You can have optional values that belong to the group inside and outer flags between multiple
groups

```run,id:1,id:2
--rect --width 10 --painted --height 10 --mirror --rect --height 10 --width 10
```

But with `adjacent` they cannot interleave

```run,id:1,id:2
--rect --rect --width 10 --painted --height 10 --height 10 --width 10
```

Or have items that don't belong to the group inside them

```run,id:1,id:2
--rect --width 10 --mirror --painted --height 10 --rect --height 10 --width 10
```