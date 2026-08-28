`--help` shows `any` parsers alongside positional items:

```console
bash$ any --help
Usage: any NUM N

Available positional items:
    NUM         Help for the first item
    N           Help for the second item

Available options:
    -h, --help  Prints help information
```

The `any` parser can consume positional arguments:

```console
bash$ any 42 10
Options { item: 42, second: 10 }
```

It can also consume items formatted like short flags (such as negative numbers):

```console
bash$ any -1 30
Options { item: -1, second: 30 }
```

However, it accepts only items that pass the `check` closure, here `-1a` fails to match the check:

```console
bash$ any -1a
expected 'NUM', and more, got '-1a'
```

Item can pass the validation, but fail in `parse`, `bpaf` outputs custom error message.

```console
bash$ any -13 130
couldn't parse '130': 130 is not in -100..100 range!
```
