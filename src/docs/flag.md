<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
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

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
# #[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(flag(Decision::Yes, Decision::No))]
    decision: Decision,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Yes,
    No,
}
```

</details>
<details>
<summary>Examples</summary>


Presense of a long name is decoded into `Yes`
```console
% app --decision
Options { decision: Yes }
```

Absense is `No`
```console
% app 
Options { decision: No }
```

</details>
