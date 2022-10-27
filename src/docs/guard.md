<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    number: u32,
}
pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").guard(
        |n| *n <= 10,
        "Values greater than 10 are only available in the DLC pack!",
    );
    construct!(Options { number }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
fn dlc_check(number: &u32) -> bool {
    *number <= 10
}

const DLC_NEEDED: &str = "Values greater than 10 are only available in the DLC pack!";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# #[allow(dead_code)]
pub struct Options {
    #[bpaf(argument("N"), guard(dlc_check, DLC_NEEDED))]
    number: u32,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


You can use guard to set boundary limits or perform other checks on parsed values.
Numbers below 10: parser accepts number below 10
```console
% app --number 5
Options { number: 5 }
```

But fails with the error message on higher values:
```console
% app --number 11
"11": Values greater than 10 are only available in the DLC pack!
```

But if function inside the parser fails - user will get the error back unless it's handled
in some way
```console
% app --number ten
Couldn't parse "ten": invalid digit found in string
```

</details>
