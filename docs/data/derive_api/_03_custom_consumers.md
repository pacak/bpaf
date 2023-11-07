#### Consumers and their customization

By default, `bpaf` picks parsers depending on a field type according to those rules:

1. `bool` fields are converted into switches: [`SimpleParser::switch`], when value is present
   it parses as `true`, when it is absent - `false`

```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(switch)]
    switch: bool,
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:1
--switch
```

```run,id:1

```



2. `()` (unit) fields, unit variants of an enum or unit structs themselves are handled as
   [`SimpleParser::req_flag`] and thus users must always specify them for the parser to succeed


```rust,id:2
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// You must agree to proceed
    agree: (),
}

fn main() {
    println!("{:?}", options().run());
}
```


```run,id:2
--help
```

```run,id:2
--agree
```

```run,id:2

```

3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but
   smartly, so non-utf8 `PathBuf`/`OsString` are working as expected.

```rust,id:3
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// numeric argument
    width: usize,
    /// IPv4 address
    addr: std::net::Ipv4Addr,
    /// Path
    path: std::path::PathBuf,
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:3
--width 42 --addr 127.0.0.1 --path /etc/passwd
```


4. For values wrapped in `Option` or `Vec` bpaf derives the inner parser and then applies
   applies logic from [`Parser::optional`] and [`Parser::many`] respectively. You can also
   use `optional` and `many` annotation explicitly.

```rust,id:4
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// optional numeric argument
    width: Option<usize>,
    /// many IPv4 addresses
    addr: Vec<std::net::Ipv4Addr>,
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:4
--addr 127.0.0.1 --addr 10.0.1.254
```

5. Fields in tuple structures are converted into positional parsers


```rust,id:5
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options (
    /// First positional
    String,
    /// second positional
    usize
);

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:5
--help
```

```run,id:5
Bob 42
```

6. You can change it with explicit annotations like `switch`, `flag`, `req_flag`, `argument` or
   `positional`. `external` annotation allows you to nest results from a whole different
   parser. `external` is somewhat special since it disables any logic that applies extra
   transformations based on the type. For example if you have an optional `external` field -
   you have to specify that it is `optional` manually.

```rust,id:6
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(switch)]
    switch: bool,

    /// Explicit required flag
    #[bpaf(req_flag(42))]
    agree: u8,

    /// Custom boolean switch with inverted values
    #[bpaf(flag(false, true))]
    inverted: bool,

    /// Custom argument
    #[bpaf(argument("DIST"))]
    distance: f64,

    /// external parser
    #[bpaf(external, optional)]
    rectangle: Option<Rectangle>,

    /// Custom positional number
    #[bpaf(positional("NUM"))]
    argument: usize,
}

#[derive(Debug, Clone, Bpaf)]
pub struct Rectangle {
    /// Width of the rectangle
    width: usize,
    /// Height of the rectangle
    height: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:6
--help
```

```run,id:6
--switch --agree --inverted --distance 23 --width 20 --height 30 42
```

With arguments that consume a value you can specify its type using turbofish syntax

```rust,id:12
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom argument
    #[bpaf(positional::<usize>("LENGTH"))]
    argument: usize,
}

fn main() {
    let opts = options().run();
    println!("{:?}", opts)
}
```

```run,id:12
42
```
