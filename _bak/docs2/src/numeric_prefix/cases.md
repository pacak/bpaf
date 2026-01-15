If `bpaf` can parse first positional argument as number - it becomes a numeric prefix

> 10 eat

Otherwise it gets ignored

> "just eat"


If validation passes but second argument is missing - in this example there's no fallback

> 10

Help should show that the prefix is optional

> --help
