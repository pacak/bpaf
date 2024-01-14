#### Argument parser

Next in complexity would be a parser to consume a named argument, such as `-p my_crate`. Same
as with the switch parser it starts from a `SimpleParser<Named>` but the next method is
[`SimpleParser::argument`]. This method takes a metavariable name - a short description that
will be used in the `--help` output. `rustc` also needs to know the parameter type you are
trying to parse, there are several ways to do it:

```rust,id:1
# use bpaf::*;
fn simple_argument_1() -> impl Parser<String> {
    // rustc figures out the type from returned value
    long("name").help("Crate name").argument("String")
}

fn simple_argument_2() -> impl Parser<String> {
    // type is specified explicitly with turbofish
    long("name").help("Crate name").argument::<String>("NAME")
}

fn main() {
    println!("{:?}", simple_argument_2().run());
}
# pub fn options() -> OptionParser<String> { simple_argument_2().to_options() }
```

```run,id:1
--name my_crate
```

```run,id:1
--help
```

You can use any type for as long as it implements [`FromStr`](std::str::FromStr). To parse
items that don't implement it you can first parse a `String` or `OsString` and then use
[`Parser::parse`], see the next chapter on how to do that.

Unlike [`SimpleParser::switch`], by default parser for argument requires it to be present on a
command line to succeed. There's several ways to add a value to fallback to, for example
[`Parser::fallback`].

```run,id:1

```
