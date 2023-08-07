#### Improving the user experience

Once you have the final parser done there's still a few ways you can improve user experience.
[`OptionParser`] comes equipped with a few methods that let you set version number,
description, help message header and footer and so on.

#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_to_options.md"))]

There's a few other things you can do:

- group some of the primitive parsers into logical blocks for `--help` message with
  [`Parser::group_help`]
- add tests to make sure important combinations are handled the way they supposed to
  after any future refactors with [`OptionParser::run_inner`]
- add a test to make sure that bpaf internal invariants are satisfied with
  [`OptionParser::check_invariants`]
- generate user documentation in manpage and markdown formats with
  [`OptionParser::render_manpage`] and [`OptionParser::render_markdown`]
