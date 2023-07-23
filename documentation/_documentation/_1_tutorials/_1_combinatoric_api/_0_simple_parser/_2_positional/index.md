#### Positional item parser

And the last simple option type is parser for positional items. Since there's no name you use
[`positional`] method directly which behaves similarly to [`NamedArg::argument`] - takes
metavariable name and a type parameter in some form. You can also attach help message directly
to it thanks to [`ParsePositional::help`]

Full example:
#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_positional.md"))]
