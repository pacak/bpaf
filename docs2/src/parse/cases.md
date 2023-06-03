`parse` don't make any changes to generated `--help` message

> --help

You can use `parse` to apply arbitrary failing transformation to any input.
For example here `--number` takes a numerical value and doubles it

> --number 10

But if function inside the parser fails - user will get the error back unless it's handled
in some other way

> --number ten
