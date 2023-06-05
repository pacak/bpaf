`--help` message describes all the flags as expected

> --help

Parser obeys the defaults

>

And can handle custom values

> --turbo -xinerama +backing

`bpaf` won't be able to generate good error messages or suggest to fix typos to users since it
doesn't really knows what the function inside `any` is going to consume

> --turbo -xinerama +backin
