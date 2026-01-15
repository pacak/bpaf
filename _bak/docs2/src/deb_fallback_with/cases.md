`fallback_with` changes parser to fallback to a value that comes from a potentially failing
computation when argument is not specified

>

If value is present - fallback value is ignored

> --version 10

Parsing errors are preserved and presented to the user

> --version ten

`bpaf` encases parsers with fallback value of some sort in usage with `[]`

> --help
