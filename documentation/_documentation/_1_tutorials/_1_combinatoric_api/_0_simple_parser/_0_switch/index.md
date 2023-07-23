#### Switch parser

Let's start with the simpliest possible one - a simple switch that gets parsed into a `bool`.

First of all - switch needs a name - you can start with [`short`] or [`long`] and add more
names if you want: `long("simple")` or `short('s').long("simple")`. This gives something with
type [`NamedArg`]:

```rust
# use bpaf::*;
use bpaf::parsers::NamedArg;
fn simple_switch() -> NamedArg {
    short('s').long("simple")
}
```

From `NamedArg` you make a switch parser by calling [`NamedArg::switch`]. Usually you do it
right away without assigning `NamedArg` to a variable.

```rust
# use bpaf::*;
fn simple_switch() -> impl Parser<bool> {
    short('s').long("simple").switch()
}
```

Switch parser we just implements trait [`Parser`] and to run it you convert it to [`OptionParser`] with
[`Parser::to_options`] and run it with [`OptionParser::run`]

Full example with some sample inputs and outputs, click to open
#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_switch.md"))]


With [`NamedArg::help`] you can attach a help message that will be used in `--help` output.
