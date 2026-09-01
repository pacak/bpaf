Add names using [`Self::short()`](Literal::short) and
[`Self::long()`](Literal::long) methods.

Parser can have many short and long names. `bpaf` shows first short and first
long names in the `--help` output; remaining names act as hidden aliases.

Specify all the names then convert the builder into a [`Parser`] using one of:

- [`Self::flag(active, default)`](Literal::flag) - a `flag` that switches from
  a "default value" to an "active value" when encountered. For a boolean value,
  use `switch` instead.
- [`Self::switch()`](Literal::switch) - a version of a `flag` that uses `true`
  and `false`.
- [`Self::req_flag(active)`](Literal::req_flag) - same as `flag`, but with no
  default value. In particular, this `flag` never parses successfully by
  itself.
- [`Self::nest(inner)`](Literal::nest) - runs the `inner` `Parser` after
  consuming the matching `literal`. Allows creating a
  [`command`](OptionParser::command)-like parser that doesn't have a separate
  help page and allows chaining by default.

Note: `literal` parsers have the same [precedence](crate::api::precedence) as
[`positional`](crate::positional) parsers and share similar triggers, so to
allow both a `literal` and a positional parser in [concurrent
branches](crate::api::algebra) - put the `literal` parser in an earlier branch.

Note: `literal` parsers with different names can execute independently of each
other, but since no established conventions exist for displaying this in the
usage line - `bpaf` renders them as sequential. Override this with
[`custom_usage`](Parser::custom_usage).
