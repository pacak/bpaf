Like `any`, this is a positional item.

```console
bash$ any_from_str --help
Usage: any_from_str NUM RTS

Available positional items:
    NUM         Help for the first item
    RTS         Enable RTS mode with '+RTS'

Available options:
    -h, --help  Prints help information
```

On a successful parse, it returns the parsed values.


```console
bash$ any_from_str -42 +RTS
Options { item: -42, rts: Rts }
```

If `FromStr` fails, bpaf treats the item as non-matching and reports a
combination of "missing item" and "unexpected item" errors at the end of the
input:

```console
bash$ any_from_str 42 --foo
expected 'RTS', got '--foo'
```
