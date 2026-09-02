[`Positional`] parsers consume operands - command line items that don't start
with a dash. To create one specify the `metavar` for the help and, optionally,
the target type with a turbofish:

- [`positional::<T>(metavar)`](crate::positional) - parses a single operand
  into `T` using its [`FromStr`] instance

Multiple `positional` parsers in a [product](crate::api::algebra) consume
operands in the order they appear on the command line.

Items that start with a dash count as named items. To pass them as
operands, insert a `--` separator before them.

`Positional` implements several inherent methods:
- [`Self::help()`](Positional::help) adds a help message for use in `--help`.
- [`Self::strict()`](Positional::strict) - causes this `positional` parser to
  only accept values after `--`.
- [`Self::posix()`](Positional::posix) - enforces POSIX-like ordering for this
  parser: after this parser succeeds, `bpaf` treats each item as positional.

`Positional` implements the [`Parser`] trait; check its methods to further
customize the parser.
