
````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, env("USER"), argument("USER"))]
    /// Custom user name
    username: String,
}
````

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    username: String,
}

pub fn options() -> OptionParser<Options> {
    let username = long("username")
        .short('u')
        .env("USER")
        .help("Custom user name")
        .argument::<String>("USER");
    construct!(Options { username }).to_options()
}
````

Help message shows env variable name along with its value, if it is set



```text
$ app --help
Usage: app -u=USER

Available options:
    -u, --username=USER  Custom user name
                         [env:USER = "pacak"]
    -h, --help           Prints help information
```


When both named argument and environment variable are present - name takes the priority



```text
$ app -u bob
Options { username: "bob" }
```


Otherwise parser falls back to the environment variable or fails with a usual "value not found"
type of error if the environment variable is not set either.



```text
$ app 
Options { username: "pacak" }
```

