Help message describes all the parser combined
> --help

Users can pass value that satisfy either parser

> --miles 42
> --kilo 15

But not both at once or not at all:

> --miles 53 --kilo 10
>

If those cases are valid you can handle them with `optional` and `many`
