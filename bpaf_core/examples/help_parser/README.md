By default `bpaf` uses `-h` / `--help` with a "run twice for full help"
semantic ([`once_twice`](crate::help::once_twice)). Use
[`help_parser`](crate::OptionParser::help_parser) with
[`help::short_long`](crate::help::short_long) to change this: `-h` renders the
brief version of the help - first paragraph only, `--help` renders the full
version including every paragraph.

Use other helpers available in [`help`](crate::help) or make your own following
the guide in [`help_parser`](crate::OptionParser::help_parser).

See [`help`](crate::help) for information about paragraph splitting.
