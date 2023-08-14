#### Start making a parser with a short name


asdf

```rust,id:1
# use bpaf::*;
fn parser() -> impl Parser<bool> {
    short('s')
        .help("A custom switch with a short name")
        .switch()
}
# pub fn options() -> OptionParser<bool> { parser().to_options() }
fn main() {
    println!("{:?}", parser().run());
}
```

help message

```run,id:1
--help
```

default is false
```run,id:1

```

when passed - is true
```run,id:1
-s
```
