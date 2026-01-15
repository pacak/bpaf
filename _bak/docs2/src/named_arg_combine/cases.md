`--help` output will contain first short and first long names that are present and won't have
anything about hidden aliases.

> --help

`--detailed` is a hidden alias and still works despite not being present in `--help` output
above

> -o -s 2 --detailed

And hidden means actually hidden. While error message can suggest to fix a typo to make it a
valid _visible_ argument

> -o best.txt -s 10 --verbos

It will not do so for hidden aliases

> -o best.txt -s 10 --detaile


In this example names `-o` and `--output` can be parsed by two parsers - `to_file` and
`to_console`, first one succeeds only if `-o` is followed by a non option name, `best.txt`.

> -o best.txt --size 10

If such name is not present - parser will try to consume one without, producing `ToConsole`
variant.

> -o -s 42

If neither is present - it fails - parser for `output` expects one of its branches to succeed

> -s 330

But this can be fixed with [`optional`](Parser::optional) (not included in this example).
