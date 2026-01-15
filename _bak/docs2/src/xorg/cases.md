`xorg` takes parameters in a few different ways, notably as a long name starting with plus or
minus with different defaults

> -xinerama +backing

But also as `+ext name` and `-ext name` to enable or disable an extensions

> --turbo +ext banana -ext apple

While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.

> --help
