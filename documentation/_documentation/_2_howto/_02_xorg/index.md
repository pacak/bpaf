#### `Xorg(1)`: `Xorg +xinerama +extension name`

This example implements syntax similar to used by `Xorg` or similar programs. As usual with
strange examples [`any`] serves an important role.

Example implements following parsers:

- enable or disable an extension using `+ext name` and `-ext name` like syntax
- enable or disable specific extension with syntax like `-xinerama` or `+backing`

Both parsers use [`any`] with [`ParseAny::anywhere`]


#![cfg_attr(not(doctest), doc = include_str!("docs2/xorg.md"))]
