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

<div style="padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;">

**Usage**: \[**`-s`**\]

**Available options:**
- **`-s`** &mdash; 
  A custom switch with a short name
- **`-h`**, **`--help`** &mdash; 
  Prints help information



</div>

default is false

<div style="padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;">
false
</div>

when passed - is true

<div style="padding: 14px; background-color:var(--code-block-background-color); font-family: 'Source Code Pro', monospace; margin-bottom: 0.75em;">
true
</div>

