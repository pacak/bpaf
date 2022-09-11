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
    On,
    Off,
    Undecided,
}

// user can specify either --on or --off, parser would fallback to `Undecided`
fn parse_decision() -> impl Parser<Decision> {
    let on = long("on").help("Positive decision").req_flag(Decision::On);
    let off = long("off")
        .help("Negative decision")
        .req_flag(Decision::Off);
    construct!([on, off]).fallback(Decision::Undecided)
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
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external)]
    decision: Decision,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(fallback(Decision::Undecided))]
pub enum Decision {
    /// Positive decision
    On,
    /// Negative decision
    Off,
    #[bpaf(skip)]
    Undecided,
}
```

</details>
<details>
<summary>Examples</summary>


This example implements a tri-state switch: wether decision was made
positive, negative or not at all. Alternative implementation can use
[`optional`](Parser::optional) and `None` to indicate "no decision" case.
A case with positive decision:
```console
% app --on
Options { decision: On }
```

A case with no decision:
```console
% app 
Options { decision: Undecided }
```

`--on` and `--off` are mutually exclusive:
```console
% app --on --off
--off is not expected in this context
```

help
```console
% app --help
Usage: [--on | --off]

Available options:
        --on    Positive decision
        --off   Negative decision
    -h, --help  Prints help information
```

</details>
