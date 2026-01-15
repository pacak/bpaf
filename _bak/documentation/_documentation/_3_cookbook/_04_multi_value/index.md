#### Multi-value arguments: `--foo ARG1 ARG2 ARG3`

By default arguments take at most one value, you can create multi value options by using
[`adjacent`](crate::parsers::ParseCon::adjacent) modifier

#![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_struct_0.md"))]
