Add more names using [`.short()`](Named::short) and [`.long()`](Named::long)
methods or more environment variables using [`.env()`](Named::env).

Parser can have many short and long names. `bpaf` shows first short and first long names
in the `--help` output; the rest are hidden aliases.

Specify all the names then convert the builder into a [`Parser`] using one of:
- [`Self::argument::<T>(meta)`](Named::argument) - an option taking a
  parameter/value
- [`Self::flag(active, default)`](Named::flag) - a flag that switches from a
  "default value" to an "active value" when encountered. For a boolean value,
  use `switch` instead.
- [`Self::switch()`](Named::switch) - a version of a `flag` that uses `true`
  and `false`.
- [`Self::req_flag(active)`](Named::req_flag) - same as `flag`, but with no
  default value. In particular, this flag never parses successfully by itself.
- [`Self::nest(inner)`](Named::nest) - similar to `argument`, but instead of
  consuming a single value run an inner parser.

`flag` variants parse as "active value" when either a configured name is present
or a configured environment variable is set, regardless of the variable
contents. `argument` uses the value itself.

Multiple parsers can share the same name. `bpaf` handles evaluation order and
result selection according to [precedence order](crate::api::precedence).
