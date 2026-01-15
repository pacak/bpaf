`bpaf` generates usual help message with

> --help

Unlike usual application `dd` takes it arguments in shape of operations
`KEY=VAL` without any dashes, plus usual `--help` and `--version` flags.

To handle that we define custom basic parsers that make handling such operations easy

> if=/dev/zero of=/tmp/blob bs=1024
