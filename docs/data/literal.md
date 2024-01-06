```rust,id:1
# use bpaf::*;
fn turbo() -> impl Parser<bool> {
    literal("+turbo", true)
        .anywhere()
        .help("Engage turbo mode!")
        // it is important to specify fallback after you done customizing literal
        // part of the parser since it gives you something other than SimpleParser
        .fallback(false)
}

fn main() {
    println!("{:?}", turbo().run());
}
# pub fn options() -> OptionParser<bool> { turbo().to_options() }
```

This parser looks for a string literal `+turbo` anywhere on the command line and produces
`true` if it was found

```run,id:1
+turbo
```

and `false otherwise

```run,id:1

```

Help message reflects this

```run,id:1
--help
```


Currently there's no way to derive `literal` parsers directly, but you can use
`external` to achieve the same result

```rust,id:2
# use bpaf::*;
fn turbo() -> impl Parser<bool> {
    literal("+turbo", true)
        .anywhere()
        .help("Engage turbo mode!")
        // it is important to specify fallback after you done customizing literal
        // part of the parser since it gives you something other than SimpleParser
        .fallback(false)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# pub
struct Options {
    #[bpaf(external)]
    turbo: bool
}

fn main() {
    println!("{:?}", options().run());
}
```

```run,id:2
+turbo
```
