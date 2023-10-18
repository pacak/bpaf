
````rust
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("SPEC"), adjacent)]
    /// Package to use
    package: String,
}
````

````rust
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    package: String,
}

fn package() -> impl Parser<String> {
    long("package")
        .short('p')
        .help("Package to use")
        .argument("SPEC")
        .adjacent()
}

pub fn options() -> OptionParser<Options> {
    construct!(Options { package() }).to_options()
}
````



```text
$ app --help
Usage: app -p=SPEC

Available options:
    -p, --package=SPEC  Package to use
    -h, --help          Prints help information
```


As with regular [`argument`](SimpleParser::argument) its `adjacent` variant is required by default



```text
$ app 
Error: expected `--package=SPEC`, pass `--help` for usage information
```


But unlike regular variant `adjacent` requires name and value to be separated by `=` only



```text
$ app -p=htb
Options { package: "htb" }
```



```text
$ app --package=bpaf
Options { package: "bpaf" }
```


Separating them by space results in parse failure



```text
$ app --package htb
Error: expected `--package=SPEC`, got `--package`. Pass `--help` for usage information
```



```text
$ app -p htb
Error: expected `--package=SPEC`, got `-p`. Pass `--help` for usage information
```



```text
$ app --package
Error: expected `--package=SPEC`, got `--package`. Pass `--help` for usage information
```

