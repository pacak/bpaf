#### Making nested parsers

Up to this point we've been looking at cases where fields of a structure are all simple
parsers, possibly wrapped in `Option` or `Vec`, but it also possible to nest derived parsers
too:

#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_nesting.md"))]
