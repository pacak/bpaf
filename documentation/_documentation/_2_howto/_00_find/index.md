#### `find(1)`: `find -exec commands -flags terminated by \;`

Example for `find` shows how to implement 3 different unusual options:

- an option with a long name but single dash as a prefix: `-user bob`
- an option that captures everything until the next fixed character
- an option that takes a set of characters: `-mode -rw`, `mode /rw`

In all cases long name with a single dash is implementedby [`literal`] with
[`ParseAny::anywhere`](crate::parsers::ParseAny::anywhere) with some items made `adjacent` to it.

To parse `-user bob` this is simply literal `-user` adjacent to a positional item with `map` to
focus on the interesting part.

For `-exec more things here ;` this is a combination of literal `-exec`, followed by `many`
items that are not `;` parsed positionally with `any` followed by `;` - again with `any`, but
`literal` works too.

And lastly to parse mode - after the tag we accept `any` to be able to handle combination of
modes that may or may not start with `-` and use [`Parser::parse`] to parse them or fail.

All special cases are made optional with [`Parser::optional`], but [`Parser::fallback`] also
works.

#![cfg_attr(not(doctest), doc = include_str!("docs2/find.md"))]
