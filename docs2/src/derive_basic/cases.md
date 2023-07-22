> --help

`--help` shows arguments as a short name with attached metavariable

Value can be separated from flag by space, `=` sign

> --name Bob --age 12
> --name "Bob" --age=12
> --name=Bob
> --name="Bob"

Or in case of short name - be directly adjacent to it

> -nBob

For long names - this doesn't work since parser can't tell where name
stops and argument begins:

> --age12

Either way - value is required, passing just the argument name results in parse failure

> --name
