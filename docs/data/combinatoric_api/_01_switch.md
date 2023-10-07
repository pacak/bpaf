#### Switch parser

Let's start with the simplest possible one - a simple switch that gets parsed into a `bool`.

First of all - the switch needs a name - you can start with [`short`] or [`long`] and add more
names if you want: `long("simple")` or `short('s').long("simple")`. This gives something with
the type [`SimpleParser`]:

```rust
# use bpaf::*;
use bpaf::{SimpleParser, parsers::Named};
fn simple_switch() -> SimpleParser<Named> {
    short('s').long("simple")
}
```

With [`SimpleParser::help`] you can attach a help message that will be used in `--help` output.

From `SimpleParser` you make a switch parser by calling [`SimpleParser::switch`]. Usually, you do it
right away without assigning `SimpleParser` to a variable.

```rust,id:1
# use bpaf::*;
fn simple_switch() -> impl Parser<bool> {
    short('s').long("simple").help("A simple switch").switch()
}

fn main() {
    println!("{:?}", simple_switch().run());
}
# pub fn options() -> OptionParser<bool> { simple_switch().to_options() }
```


The switch parser we just made implements trait [`Parser`]. You can run it right right away
with [`Parser::run`] or convert to [`OptionParser`] with [`Parser::to_options`] and run it with
[`OptionParser::run`]. Later allows attaching extra help information.


```run,id:1
--simple
```

When switch is not present on a command line - parser produces `false`.

```run,id:1

```

You also get a help message.

```run,id:1
--help
```
