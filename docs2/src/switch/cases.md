In `--help` output `bpaf` shows switches as usual flags with no meta variable attached

> --help

Both `switch` and `flag` succeed if value is not present, `switch` returns `false`, `flag` returns
second value.

>

When value is present - `switch` returns `true`, `flag` returns first value.

> --verbose --no-default-features --detailed

Like with most parsrs unless specified `switch` and `flag` consume at most one item from the
command line:

> --no-default-features --no-default-features
