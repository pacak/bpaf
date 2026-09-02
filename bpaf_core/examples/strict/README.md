A regular [`Positional`] parser accepts the next operand wherever it appears on
the command line. Calling [`Positional::strict`](Positional::strict) changes
this behavior: the parser only accepts operands that follow a `--` separator.

The example combines a regular `positional` parser with a `strict` one. The
first operand `ALPHA` accepts a value anywhere on the command line. The second
operand `BETA` accepts a value only after a `--` separator.