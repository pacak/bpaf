## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    desert: Option<&'static str>,
}

pub fn options() -> OptionParser<Options> {
    let desert = ["apple", "banana", "orange", "grape", "strawberry"]
        .iter()
        .map(|name| {
            long(name)
                .help("Pick one of the options")
                .req_flag(*name)
                .boxed()
        });
    let desert = choice(desert).optional();
    construct!(Options { desert }).to_options()
}
````

Here [`choice`](choice) function is used to create an option for each possible desert item



```text
$ app --help
Usage: app [--apple | --banana | --orange | --grape | --strawberry]

Available options:
        --apple       Pick one of the options
        --banana      Pick one of the options
        --orange      Pick one of the options
        --grape       Pick one of the options
        --strawberry  Pick one of the options
    -h, --help        Prints help information
```


User can pick any item



```text
$ app --apple
Options { desert: Some("apple") }
```


Since parser consumes only one value you can't specify multiple flags of the same type



```text
$ app --orange --grape
Error: `--grape` cannot be used at the same time as `--orange`
```


And [`Parser::optional`](Parser::optional) makes it so when value is not specified - `None` is produced instead



```text
$ app 
Options { desert: None }
```

