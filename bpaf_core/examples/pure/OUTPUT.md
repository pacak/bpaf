`pure` does not show up in the `--help` message

```console
bash$ pure --help
Usage: pure --name=NAME

Available options:
        --name=NAME  Use a custom user name
    -h, --help       Prints help information
```

It produces a value that is hardcoded in the parser

```console
bash$ pure --name Bob
Options { name: "Bob", money: 330 }
```

There is no way to alter the value from the command line

```console
bash$ pure --money 100000 --name Hackerman
expected '--name=NAME', got '--money'
```
