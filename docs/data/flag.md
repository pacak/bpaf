## Derive example

```rust,id:1
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
```

## Combinatoric example

```rust,id:2
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
```


In `--help` output `bpaf` shows flags with no meta variable attached

```run,id:1,id:2
--help
```

Presense of a long name is decoded into `Yes`

```run,id:1,id:2
--decision
```

Absense is `No`

```run,id:1,id:2

```
