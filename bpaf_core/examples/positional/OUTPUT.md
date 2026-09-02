`--help` shows the metavar for each positional item:

```console
bash$ positional --help
Usage: positional ALPHA BETA

Available positional items:
    ALPHA       Help for argument alpha
    BETA        Help for argument beta

Available options:
    -h, --help  Prints help information
```

`bpaf` consumes operands in order, parsing each using the target type's
[`FromStr`](std::str::FromStr) instance.

```console
bash$ positional 42 hello
Options { alpha: 42, beta: "hello" }
```

By default there is no fallback value

```console
bash$ positional 42
expected 'BETA'
```

Items that fail to parse into the target type produce an error:

```console
bash$ positional banana 42
couldn't parse 'banana': invalid digit found in string
```

`bpaf` treats items that start with a dash as named items, so the `positional`
parser does not see them:

```console
bash$ positional 42 --foo
expected 'BETA', got '--foo'
```

Use the `--` separator to convert all the remaining items into `positional` items.

```console
bash$ positional 42 -- -foo
Options { alpha: 42, beta: "-foo" }
```
