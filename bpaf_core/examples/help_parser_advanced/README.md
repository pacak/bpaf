To work with `help_parser`, the parser must produce variants of
[`Help`](crate::help::Help) `enum`, `Help::Brief` for the brief version and
`Help::Full` for the full one and must only succeed with the input from the
user, so use
[`Named::req_flag`](crate::api::primitives::Named::req_flag) or similar methods.

This example binds `-h` and `--help` to the brief version and `--full-help` to
the full version. It also replaces its own help entry via
[`help_literal`](crate::Parser::help_literal) so the `--help` shows both long
names.
