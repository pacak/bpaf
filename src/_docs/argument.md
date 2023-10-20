## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // you can specify exact type argument should produce
    // for as long as it implements `FromStr`
    #[bpaf(short, long, argument::<String>("NAME"))]
    /// Specify user name
    name: String,
    // but often rust can figure it out from the context,
    // here age is going to be `usize`
    #[bpaf(argument("AGE"), fallback(18), display_fallback)]
    /// Specify user age
    age: usize,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
    age: usize,
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    let age = long("age")
        .help("Specify user age")
        // but often rust can figure it out from the context,
        // here age is going to be `usize`
        .argument("AGE")
        .fallback(18)
        .display_fallback();

    construct!(Options { name, age }).to_options()
}
````



```text
$ app --help
Usage: app -n=NAME [--age=AGE]

Available options:
    -n, --name=NAME  Specify user name
        --age=AGE    Specify user age
                     [default: 18]
    -h, --help       Prints help information
```


`--help` shows arguments as a short name with attached metavariable

Value can be separated from flag by space, `=` sign



```text
$ app --name Bob --age 12
Options { name: "Bob", age: 12 }
```



```text
$ app --name "Bob" --age=12
Options { name: "Bob", age: 12 }
```



```text
$ app --name=Bob
Options { name: "Bob", age: 18 }
```



```text
$ app --name="Bob"
Options { name: "Bob", age: 18 }
```


Or in case of short name - be directly adjacent to it



```text
$ app -nBob
Options { name: "Bob", age: 18 }
```


For long names - this doesn't work since parser can't tell where name
stops and argument begins:



```text
$ app --age12
Error: no such flag: `--age12`, did you mean `--age`?
```


Either way - value is required, passing just the argument name results in parse failure



```text
$ app --name
Error: `--name` requires an argument `NAME`
```

