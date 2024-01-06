#### Positional item parser

Next last simple option type is a parser for positional items. Since there's no name you use
the [`positional`] function directly. Similar to [`SimpleParser::argument`] this function takes
a metavariable name and a type parameter in some form. You can also attach the help message
thanks to [`SimpleParser::help`]

```rust,id:1
# use bpaf::*;
fn pos() -> impl Parser<String> {
    positional("URL").help("Url to open")
}

fn main() {
    println!("{:?}", pos().run());
}
# pub fn options() -> OptionParser<String> { pos().to_options() }
```

Same as with argument by default there's no fallback so with no arguments parser fails

```run,id:1

```

Other than that any name that does not start with a dash or explicitly converted to positional
parameter with `--` gets parsed:

```run,id:1
https://lemmyrs.org
```

```run,id:1
"strange url"
```

```run,id:1
-- --can-start-with-dash-too
```

And as usual there's help message

```run,id:1
--help
```
