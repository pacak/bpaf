#### Making nested parsers

Up to this point, we've been looking at cases where fields of a structure are all simple
parsers, possibly wrapped in `Option` or `Vec`, but it is also possible to nest derived parsers
too:

#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_nesting.md"))]


`external` takes an optional function name and will call that function to make the parser for
the field. You can chain more transformations after the `external` and if the name is absent -
`bpaf` would use the field name instead, so you can also write the example above as


```rust
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
    #[bpaf(external)]
    format: Format,
}
```
