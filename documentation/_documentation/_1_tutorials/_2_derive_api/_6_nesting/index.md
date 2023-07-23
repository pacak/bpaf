#### Making nested parsers

Up to this point we've been looking at cases where fields of a structure are all simple
parsers, possibly wrapped in `Option` or `Vec`, but it also possible to nest derived parsers
too:

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub enum Format {
    /// Produce output in HTML format
    Html,
    /// Produce output in Markdown format
    Markdown,
    /// Produce output in manpage format
    Manpage
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// File to process
    input: String,
    #[bpaf(external(format))]
    format: Format
}

fn main() {
    let opts = options().run();
    println!("{:?}", opts);
}
```

`external` annotation replaces the consumer and parameter it takes is a function name created
either manually with combinatoric API or derived with `#[derive(Bpaf)]`. If parameter is
omitted then it would default to the field name. In example above since both function and field
are called `format` - annotation `#[bpaf(external)]` would be sufficient.
