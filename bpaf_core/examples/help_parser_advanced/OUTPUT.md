`-h` and `--help` show the brief help:

```console
bash$ help_parser_advanced --help
Usage: help_parser_advanced -a=A -b=B

Available options:
    -a, --alpha=A            Help for argument alpha
    -b, --beta=B             Help for argument beta
    -h, --help, --full-help  Prints help information
```

`--full-help` shows the full help with all the details:

```console
bash$ help_parser_advanced --full-help
Usage: help_parser_advanced -a=A -b=B

Available options:
    -a, --alpha=A            Help for argument alpha

                             Alpha counts things. Very carefully.
    -b, --beta=B             Help for argument beta

                             Beta counts other things.

                             Very carelessly.
    -h, --help, --full-help  Prints help information
```

A custom help parser doesn't affect the regular parsing:

```console
bash$ help_parser_advanced --alpha 42 -b 13
Options { alpha: 42, beta: 13 }
```
