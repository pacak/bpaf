`pure` produces a fixed value without consuming anything from the command
line, which makes it useful with [`construct!`](crate::construct) to fill in
parts of the structure that can't or shouldn't be set from the command line.

Because it doesn't consume anything, `pure` does not show up in the `--help`
message. Both `pure` and [`pure_with`](crate::pure_with) are designed to put
values into structures; for fallback values use [`Parser::fallback`] and
[`Parser::fallback_with`] methods instead.
