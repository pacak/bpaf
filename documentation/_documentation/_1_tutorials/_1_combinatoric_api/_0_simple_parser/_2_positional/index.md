#### Positional item parser

And the last simple option type is parser for positional items. Since there's no name you use
[`positional`] method directly. Similar to [`NamedArg::argument`] this method takes
metavariable name and a type parameter in some form. You can also attach the help message
thanks to [`ParsePositional::help`]

Full example:
#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_positional.md"))]
