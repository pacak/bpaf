#### Subcommand parsers

To make a parser for a subcommand you make an `OptionParser` for that subcommand first as if it
was the only thing your application would parse then turn it into a regular [`Parser`]
you can further compose with [`OptionParser::command`].

This gives [`ParseCommand`] back, you can add aliases or tweak the help message if you want to.

#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_command.md"))]
