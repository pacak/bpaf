`-h` shows the brief help:

```console
bash$ help_parser -h
Usage: help_parser -a=A -b=B

Available options:
    -a, --alpha=A  Help for argument alpha
    -b, --beta=B   Help for argument beta
    -h, --help     Prints help information
```

`--help` shows the full help with all the details:

```console
bash$ help_parser --help
Usage: help_parser -a=A -b=B

Available options:
    -a, --alpha=A  Help for argument alpha

                   Alpha counts things. Very carefully.
    -b, --beta=B   Help for argument beta

                   Beta counts other things.

                   Very carelessly.
    -h, --help     Prints help information
```

Selected help parser doesn't affect the regular parsing:

```console
bash$ help_parser --alpha 42 -b 13
Options { alpha: 42, beta: 13 }
```
