#### Customizing the consumers

By default, `bpaf` picks parsers depending on a field type according to those rules:

1. `bool` fields are converted into switches: [`NamedArg::switch`](crate::parsers::NamedArg::switch)
2. `()` (unit) fields, unit variants of an enum or unit structs themselves are handled as
   [`NamedArg::req_flag`](crate::parsers::NamedArg::req_flag) and thus users must always specify
   them for the parser to succeed
3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but
   smartly, so non-utf8 `PathBuf`/`OsString` are working as expected.
4. For values wrapped in `Option` or `Vec` bpaf derives the inner parser and then applies
   applies logic from [`Parser::optional`] and [`Parser::many`] respectively.

You can change it with annotations like `switch`, `argument` or `positional`



```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, switch)]
    switch: bool,

    /// Custom number
    #[bpaf(positional("NUM"))]
    argument: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

`bpaf` generates help message with a short name only as described

```run,id:1
--help
```

And accepts the short name only

```run,id:1
-s 42
```

long name is missing

```run,id:1
--switch 42
```


With arguments that consume a value you can specify its type using turbofish-line syntax

```rust,id:2
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

```run,id:2
42
```
