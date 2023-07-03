Despite parser producing a funky value - help looks like you would expect from a parser that
takes two values

> --help

When executed with no parameters it produces four `None` values - all parsers succeed by the
nature of them being [`optional`](Parser::optional)

>

When executed with expected parameters fields with `usize` get their values

> --height 100 --width 100

With incorrect value for `--height` parameter inner part of `height` parser fails, `optional`
combined with `catch` handles this failure and produces `None` without consuming value from the
command line. Parser `height_str` runs next and consumes the value as a string

> --height ten

In case of wrong `--width` - parser `width` fails, parser for `optional` sees this as a
"value is present but not correct" and propagates the error outside, execution never reaches
`width_str` parser

> --width ten
