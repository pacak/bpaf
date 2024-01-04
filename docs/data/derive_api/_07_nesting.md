#### Making nested parsers

Up to this point, we've been mostly looking at cases where fields of a structure are all simple
parsers, possibly wrapped in `Option` or `Vec`, but it is also possible to nest derived parsers
too:


```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub enum Format {
    /// Produce output in HTML format
    Html,
    /// Produce output in Markdown format
    Markdown,
    /// Produce output in manpage format
    Manpage,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// File to process
    input: String,
    #[bpaf(external(format))]
    format: Format,
}
```

Help message lists all possible options

```run,id:1
--help
```

Parser accepts one and only one value from enum in this example

```run,id:1
--input Cargo.toml --html
```

```run,id:1
--input hello
```

```run,id:1
--input hello --html --markdown
```

`external` takes an optional function name and will call that function to make the parser for
the field. You can chain more transformations after the `external` and if the name is absent -
`bpaf` would use the field name instead.

Because of the limitations of the macro system having `external` parser disables automatic
detection for `Option` or `Vec` containers so you have to specify it explicitly:

```rust,id:2
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub enum Format {
    /// Produce output in HTML format
    Html,
    /// Produce output in Markdown format
    Markdown,
    /// Produce output in manpage format
    Manpage,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// File to process
    input: String,
    #[bpaf(external(format), many)]
    format: Vec<Format>,
}
```

