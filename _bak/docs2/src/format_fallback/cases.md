`fallback` changes parser to fallback to a default value used when argument is not specified

>

If value is present - fallback value is ignored

> --log-file output.txt

Parsing errors are preserved and presented to the user

> --log-file /

With [`display_fallback`](ParseFallback::display_fallback),
[`debug_fallback`](ParseFallback::debug_fallback), and
[`format_fallback`](ParseFallback::format_fallback), you can make it so the default value
is visible in the `--help` output.

> --help
