`pure_with` does not show up in `--help` message

```console
bash$ pure_with --help
Usage: pure_with --name=NAME

Available options:
        --name=NAME  Use a custom user name
    -h, --help       Prints help information
```

It produces a value by calling a closure

```console
bash$ pure_with --name Bob
Options { name: "Bob", money: 330 }
```

There is no way to alter the value from the command line

```console
bash$ pure_with --money 100000 --name Hackerman
expected '--name=NAME', got '--money'
```
