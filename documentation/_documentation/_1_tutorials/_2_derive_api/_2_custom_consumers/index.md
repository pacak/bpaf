#### Customizing the consumers

By default `bpaf` picks parsers depending on a field type according to those rules:

1. `bool` fields are converted into switches: [`NamedArg::switch`](crate::parsers::NamedArg::switch)
2. `()` (unit) fields, unit variants of enum or unit structs themselves are handled as req_flag
   [`NamedArg::req_flag`](crate::parsers::NamedArg::req_flag) and thus users must always specify
   them for parser to succeed
3. All other types with no `Vec`/`Option` are parsed using [`FromStr`](std::str::FromStr), but in a
   smart way, so Non-utf8 `PathBuf`/`OsString` are working as expected.
4. For values wrapped in `Option` or `Vec` bpaf derives inner parser and then applies
   applies logic from [`Parser::optional`] and [`Parser::many`] respectively.

You can change it with annotations like `switch`, `argument` or `positional`


#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_custom_consumer.md"))]

With arguments that consume a value you can specify its type using turbofish-line syntax


```no_run
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
