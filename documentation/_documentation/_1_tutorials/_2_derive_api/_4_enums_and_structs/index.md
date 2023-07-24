#### Parsing structs and enums

To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
it for some other purpose as well and want to skip them during parsing you can use [`pure`].

If you use `#[derive(Bpaf)]` on enum parser will produce variant for which all the parsers
succeed.

#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_enum.md"))]
