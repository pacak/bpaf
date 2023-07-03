Despite parser producing a funky value - help looks like you would expect from a parser that
takes two values

> --help

When executed with no parameters parser fails because `some` requires you to specify at least
one matching parameter

>

When executed with expected parameters fields with `usize` get their values

> --height 100 --width 100 --height 12 --width 44

With incorrect value for `--height` parameter inner part of `height` parser fails, `some`
combined with `catch` handles this failure and produces `[]` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string

> --height 10 --height twenty --width 33

In case of wrong `--width` - parser `width` fails, parser for `some` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser

> --height 10 --width 33 --width ten
