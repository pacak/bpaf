Help output displays the primary short and long names:

```console
bash$ literal --help
Usage: literal item [f]

Available options:
    -h, --help  Prints help information

Available commands:
    item        literal required flag
    flag, f     literal optional boolean flag
```

Matching works in any order across independent literal parsers.

```console
bash$ literal item flag
Options { item: 32, flag: true }
```

```console
bash$ literal flag item
Options { item: 32, flag: true }
```

The `"flag"` parser is optional and defaults to `false` when absent.

```console
bash$ literal item
Options { item: 32, flag: false }
```

Parser for `"item"` is required.

```console
bash$ literal flag
expected 'item'
```
