`guard` don't make any changes to generated `--help` message

> --help

You can use guard to set boundary limits or perform other checks on parsed values.
Parser accepts numbers below 10

> --number 5

And fails with the error message on higher values:

> --number 11


But if function inside the parser fails - user will get the error back unless it's handled
in some way

> --number ten
