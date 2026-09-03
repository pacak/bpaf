`--help` shows the `metavar` for the positional item and the `-v` switch:

```console
bash$ posix --help
Usage: posix [-v] [FILE]...

Available positional items:
    FILE        File to process

Available options:
    -v          Emit diagnostic messages
    -h, --help  Prints help information
```

Before the parser with POSIX restriction, `bpaf` accepts named items as usual

```console
bash$ posix -v file.txt
Options { verbose: true, file: ["file.txt"] }
```

After the parser with POSIX restriction, `bpaf` treats all the remaining items
as positional operands, the same way it treats them after a `--` separator.

```console
bash$ posix file.txt -v
Options { verbose: false, file: ["file.txt", "-v"] }
```
