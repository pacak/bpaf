## Derive example

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Positive decision
    #[bpaf(flag(Decision::Yes, Decision::No))]
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}
````

## Combinatoric example

````rust
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}

fn parse_decision() -> impl Parser<Decision> {
    long("decision")
        .help("Positive decision")
        .flag(Decision::Yes, Decision::No)
}

pub fn options() -> OptionParser<Options> {
    let decision = parse_decision();
    construct!(Options { decision }).to_options()
}
````

In `--help` output `bpaf` shows flags with no meta variable attached



```text
$ app --help
Usage: app [--decision]

Available options:
        --decision  Positive decision
    -h, --help      Prints help information
```


Presense of a long name is decoded into `Yes`



```text
$ app --decision
Options { decision: Yes }
```


Absense is `No`



```text
$ app 
Options { decision: No }
```

