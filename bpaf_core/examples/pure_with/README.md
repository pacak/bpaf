`pure_with` is a possibly failing version of [`pure`](crate::pure). It produces
a value by invoking a fallible closure without consuming anything from the
command line, which makes it useful with [`construct!`](crate::construct) to
fill in parts of the structure that can't or shouldn't be set from the command line.

If the closure call fails (returns `Err(E)`) - `pure_with` fails as well.

Because it doesn't consume anything, `pure_with` does not show up in the
`--help` message. Both [`pure`](crate::pure) and `pure_with` are designed to
put values into structures; for fallback values use [`Parser::fallback`] and
[`Parser::fallback_with`] methods instead.
