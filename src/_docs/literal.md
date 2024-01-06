
````rust
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
````

This parser looks for a string literal `+turbo` anywhere on the command line and produces
`true` if it was found



```text
$ app +turbo
true
```


and \`false otherwise



```text
$ app 
false
```


Help message reflects this



```text
$ app --help
Usage: app [+turbo]

Available options:
    +turbo      Engage turbo mode!
    -h, --help  Prints help information
```


Currently there's no way to derive `literal` parsers directly, but you can use
`external` to achieve the same result

````rust
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
````



```text
$ app +turbo
Options { turbo: true }
```

