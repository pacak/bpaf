#### Start making a parser with a short name

asdf

``` rust
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

``` run

**Usage**: \[**`-s`**\]

**Available options:**
- **`-s`** &mdash;
  A custom switch with a short name
- **`-h`**, **`--help`** &mdash;
  Prints help information


```

default is false

``` run
false
```

when passed - is true

``` run
true
```

