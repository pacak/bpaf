All arguments match a registered parser: the `switch` parser handles `--flag`,
and a the `positional` parser handles `random`. There are no unparsed items:

```console
bash$ leftovers --flag random
Options { flag: true, pos: Some("random"), rest: [] }
```

`leftovers` captures unrecognized arguments such as `--random` into `rest`.

```console
bash$ leftovers random --random
Options { flag: false, pos: Some("random"), rest: ["--random"] }
```

The `switch` parser accepts at most one `--flag` (it has no
[repetition](crate::api::repeat) modifiers such as [`count`](Parser::count)),
so `leftovers` captures the second `--flag` as a leftover alongside unknown
arguments:


```console
bash$ leftovers random --flag --flag --random
Options { flag: true, pos: Some("random"), rest: ["--flag", "--random"] }
```

## Important: Parsing Halts at the First Leftover

Once `leftovers` encounters an unrecognized option, it captures all remaining
arguments. Standard parsing cannot resume after an unrecognized token like
`--random` because `bpaf` cannot determine whether `--random` was intended as a
standalone flag or an option requiring a `value`:

```console
bash$ leftovers --random value --flag
Options { flag: false, pos: None, rest: ["--random", "value", "--flag"] }
```
