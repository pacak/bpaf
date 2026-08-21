Help contains the first short and first long names, and the first environment
variable (name only) if present.

```console
bash$ named --help
Usage: named -a=A -b=B --magic=C

Available options:
    -a, --alpha=A  Help for argument alpha
    -b, --beta=B   Help for argument beta
        --magic=C  Help for flag gamma
                   Uses environment variable MAGIC
    -h, --help     Prints help information
```

Visible aliases work

```console
bash$ named --alpha 42 -b 13 --magic 1
Options { alpha: 42, beta: 13, gamma: 1 }
```

As well as environment variables

```console
bash$ MAGIC=42 named --alpha 42 -b 13
Options { alpha: 42, beta: 13, gamma: 42 }
```

`bpaf` uses environment variables as a fallback option, so the `--magic 2` value takes priority over `MAGIC=42`

```console
bash$ MAGIC=42 named --alpha 42 -b 131313  --magic 2
Options { alpha: 42, beta: 131313, gamma: 2 }
```

Hidden aliases for environment variables also work

```console
bash$ MORE_MAGIC=1 named --alpha 42 -b 131313  --magic 3
Options { alpha: 42, beta: 131313, gamma: 3 }
```
