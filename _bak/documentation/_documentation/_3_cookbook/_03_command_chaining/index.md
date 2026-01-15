#### Command chaining
Lets you do things like `setup.py sdist bdist`: [command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining)

With [`adjacent`](crate::parsers::ParseCommand::adjacent)
`bpaf` allows you to have several commands side by side instead of being nested.

#![cfg_attr(not(doctest), doc = include_str!("docs2/adjacent_command.md"))]
