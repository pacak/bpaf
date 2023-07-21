By default all arguments are required so running with no arguments produces an error

>

Bpaf accepts various combinations of names and adjacencies:

> -s100
> --size 300
> -s=42
> --size=14

Since not every string is a valid number - bpaf would report any parsing failures to the user
directly

> --size fifty

In addition to the switch you defined `bpaf` generates switch for user help which will include
the description from the `help` method

> --help
