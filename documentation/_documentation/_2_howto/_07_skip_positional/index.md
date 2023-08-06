#### Skipping optional positional items if parsing or validation fails

Combinations like [`Parser::optional`] and
[`ParseOptional::catch`](crate::parsers::ParseOptional::catch) allow to try to parse something
and then handle the error as if pase attempt never existed

#![cfg_attr(not(doctest), doc = include_str!("docs2/numeric_prefix.md"))]
