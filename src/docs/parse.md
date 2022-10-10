<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# use std::{num::ParseIntError, str::FromStr};
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    number: u32,
}
pub fn options() -> OptionParser<Options> {
    let number = long("number")
        .argument::<String>("N")
        // normally you'd use argument::<u32> and `map`
        .parse::<_, _, ParseIntError>(|s| Ok(u32::from_str(&s)? * 2));
    construct!(Options { number }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
# use std::{num::ParseIntError, str::FromStr};
fn twice_the_num(s: String) -> Result<u32, ParseIntError> {
    Ok(u32::from_str(&s)? * 2)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# #[allow(dead_code)]
pub struct Options {
    #[bpaf(argument::<String>("N"), parse(twice_the_num))]
    number: u32,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


You can use `parse` to apply arbitrary failing transformation to any input.
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
