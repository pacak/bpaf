#### Argument parser

Next in complexity would be a parser to consume a named argument, such as `-p my_crate`. Same
as with switch parser it starts from a `NamedArg` but next method is [`NamedArg::argument`].
Method takes a metavariable name - a short description that will be used in the `--help`
output. `rustc` also needs to know the type of a variable you are trying to parse, there's
several ways to do it:

```rust
# use bpaf::*;
# use std::path::PathBuf;
fn simple_argument_1() -> impl Parser<u32> {
    long("number").argument("NUM")
}

fn simple_argument_2() -> impl Parser<String> {
    long("name").argument::<String>("NAME")
}

fn file_parser() -> OptionParser<PathBuf> {
    long("file").argument::<PathBuf>("FILE").to_options()
}
```

You can use any type for as long as it implements [`FromStr`]. To parse items that don't
implement it you can first parse a `String` or `OsString` and then use [`Parser::parse`], see
[the next chapter](super::super::_1_chaining) how to do that.

Full example with some sample inputs and outputs:
#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_argument.md"))]
