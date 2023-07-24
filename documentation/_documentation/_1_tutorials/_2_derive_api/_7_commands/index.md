#### Parsing subcommands

Easiest way to define a group of subcommands is to have them inside the same enum with variant
constructors annotated with `#[bpaf(command("name"))]` with or without the name


#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_commands.md"))]
