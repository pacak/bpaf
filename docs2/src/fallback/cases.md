`fallback` changes parser to fallback to a default value used when argument is not specified

>

If value is present - fallback value is ignored

> --jobs 10

Parsing errors are preserved and presented to the user

> --jobs ten

With [`display_fallback`](ParseFallback::display_fallback) and
[`debug_fallback`](ParseFallback::debug_fallback) you can make it so default value
is visible in `--help` output

> --help
