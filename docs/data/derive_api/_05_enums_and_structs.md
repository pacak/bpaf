#### Parsing structs and enums

To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
it for some other purpose as well and want to skip them during parsing you can use [`pure`] to
fill in values in member fields and `#[bpaf(skip)]` on enum variants you want to ignore, see
combinatoric example in [`Parser::last`].

If you use `#[derive(Bpaf)]` on an enum parser will produce a variant for which all the parsers
succeed.


```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// User name
    user: String,
    #[bpaf(pure(100))]
    starting_money: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:1
--help
```

`starting_money` is filled from [`pure`] and there's no way for user to override it

```run,id:1
--user Bob
```

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    ByPath {
        path: std::path::PathBuf
    },
    ByName {
        name: String,
    },
    #[bpaf(skip)]
    Resolved {
        id: usize,
    }
}

fn main() {
    println!("{:?}", options().run());
}
```

`bpaf` ignores `Options::Resolved` constructor

```run,id:2
--help
```

```run,id:2
--name hackerman
```
