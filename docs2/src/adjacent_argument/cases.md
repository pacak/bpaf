> --help

As with regular [`argument`](NamedArg::argument) its `adjacent` variant is required by default

>

But unlike regular variant `adjacent` requires name and value to be separated by `=` only

> -p=htb
> --package=bpaf

Separating them by space results in parse failure

> --package htb
> -p htb
> --package
