When `--help` used once it renders shoter version of the help information

> --help

When used twice - it renders full version. Documentation generator uses full
version as well

> --help --help

Presence or absense of a help message should not affect the parser's output

> --name Bob output.txt
