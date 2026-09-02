`--help` shows the `metavar` for each positional item. The `--` marker in front
of `BETA` shows that the strict parser only accepts operands after the `--`
separator:

```console
bash$ strict --help
Usage: strict ALPHA -- BETA

Available positional items:
    ALPHA       Help for argument alpha
    BETA        Help for argument beta

Available options:
    -h, --help  Prints help information
```

`ALPHA` accepts the next operand wherever it appears on the command line.
`BETA` requires a `--` separator before it, so passing both operands without a
separator fails:

```console
bash$ strict 42 hello
expected 'hello' (BETA) to follow '--'
```

Place `--` between the two operands: `ALPHA` consumes the first operand and
`BETA` takes an operand from after the separator:

```console
bash$ strict 42 -- hello
Options { alpha: 42, beta: "hello" }
```

Put `--` in front of both operands: `BETA` still accepts a value after the
separator and `ALPHA` accepts an operand anywhere:

```console
bash$ strict -- 42 hello
Options { alpha: 42, beta: "hello" }
```
