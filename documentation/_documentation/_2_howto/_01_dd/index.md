#### `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`

This example implements syntax similar to `dd` command. Main idea is to implement something to
make it simple to make parsers for `PREFIX=SUFFIX`, where prefix is fixed for each parser - for
example `if=` or `of=` and suffix is parsed with usual [`FromStr`](std::str::FromStr) trait.

Function `tag` serves this purpose. It contains following steps:

- consume any item that starts with a prefix at any argument position with [`any`] and
  [`ParseAny::anywhere`]
- attaches help message and custom metadata to make `--help` friendlier
- parses suffix with [`Parser::parse`]

The rest of the parser simply uses `tag` to parse a few of `dd` arguments

#![cfg_attr(not(doctest), doc = include_str!("docs2/dd.md"))]
