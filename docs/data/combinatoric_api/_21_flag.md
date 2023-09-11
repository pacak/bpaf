#### `flag` - general version of `switch`

`bpaf` contains a few more primitive parsers: [`NamedArg::flag`] and [`NamedArg::req_flag`].
First one is a more general case of [`NamedArg::switch`] that lets you to make a parser for a
flag, but instead of producing `true` or `false` it can produce one of two values of the same
type.


```rust,id:1
# use bpaf::*;
fn simple_switch() -> impl Parser<u8> {
    short('s').long("simple").help("A simple flag ").flag(1, 0)
}

fn main() {
    println!("{:?}", simple_switch().run());
}
# pub fn options() -> OptionParser<u8> { simple_switch().to_options() }
```

```run,id:1
--simple
```

```run,id:1

```

Usually you'd use [`NamedArg::flag`] to crate an inverted switch like `--no-logging` that would
return `false` when switch is present and true otherwise. But it can also give you types
identical to `bool` but with more meaning: `Logging::Enabled` / `Logging::Disabled`, etc.
