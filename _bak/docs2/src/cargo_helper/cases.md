Let's say the goal is to parse an argument and a switch:

> --argument 15

But when used as a `cargo` subcommand, cargo will also pass the command name. For example
you can invoke an app with binary name `cargo-asm`

```console
$ cargo asm --lib --everything
...
```

`cargo` will then spawn the executable and pass it following parameters:

```console
$ cargo-asm asm --lib --everything
...
```

If you are not using `cargo_helper` - parser won't know what to do with `asm` part.
`cargo-helper` allows the parser to strip it from the front and everything works as expected.

And it doesn't show up in `--help` so not to confuse users

> --help
