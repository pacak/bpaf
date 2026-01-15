`map` don't make any changes to generated `--help` message


You can use `map` to apply arbitrary pure transformation to any input.
Here `--number` takes a numerical value and doubles it

> --number 10

But if function inside the parser fails - user will get the error back unless it's handled
in some way. In fact here execution never reaches `map` function -
[`argument`](NamedArg::argument) tries to parse `ten` as a number, fails and reports the error

> --number ten
