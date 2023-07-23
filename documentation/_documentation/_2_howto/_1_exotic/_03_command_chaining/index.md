#### [Command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining): `setup.py sdist bdist`

With [`adjacent`](crate::parsers::ParseCommand::adjacent)
`bpaf` allows you to have several commands side by side instead of being nested.

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_2.md"))]
