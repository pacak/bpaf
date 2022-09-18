<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    decision: bool,
}

fn parse_decision() -> impl Parser<bool> {
    long("decision").help("Positive decision").switch()
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
    decision: bool,
}
```

</details>
<details>
<summary>Examples</summary>


Presense of a long name is decoded into `true`
```console
% app --decision
Options { decision: true }
```

Absense is `false`
```console
% app 
Options { decision: false }
```

</details>
