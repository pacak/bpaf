`with_group_help` lets you write longer description for group of options that can also refer to
those options. Similar to [`group_help`](Parser::group_help) encased optios are separated from
the rest by a blank line.

Invoking help with a single `--help` flag renders shot(er) version of the help message
that contanis only the first paragraph for each block:

> --help

Invoking help with double `--help --help` flag renders the full help message with all the
descriptions added

> --help --help

Other than rendering the help message that there's no interactions with other parsers

> --width 120 --height 11

> --argument 12
