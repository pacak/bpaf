#### Parsing structs and enums

To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
it for some other purpose as well and want to skip them during parsing you can use [`pure`] to
fill in values in member fields and `#[bpaf(skip)]` on enum variants you want to ignore, see
combinatoric example in [`Parser::last`].

If you use `#[derive(Bpaf)]` on an enum parser will produce a variant for which all the parsers
succeed.

#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_enum.md"))]
