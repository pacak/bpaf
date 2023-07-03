In addition to all the arguments specified by user `bpaf` adds a few more. One of them is
`--help`:

> --help

The other one is `--version` - passing a string literal or something like
`env!("CARGO_PKG_VERSION")` to get version from `cargo` directly usually works

> --version

Other than that `bpaf` tries its best to provide a helpful error messages

>

And if all parsers are satisfied [`run`](OptionParser::run) produces the result

> -i 10
