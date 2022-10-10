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
    let number = long("number").argument::<u32>("N").map(|x| x * 2);
    construct!(Options { number }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
fn twice_the_num(n: u32) -> u32 {
    n * 2
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# #[allow(dead_code)]
pub struct Options {
    #[bpaf(argument::<u32>("N"), map(twice_the_num))]
    number: u32,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


You can use `map` to apply arbitrary pure transformation to any input.
normally `--number` takes a numerical value and doubles it
```console
% app --number 10
Options { number: 20 }
```

But if function inside the parser fails - user will get the error back unless it's handled
in some way
```console
% app --number ten
Couldn't parse "ten": invalid digit found in string
```

</details>
