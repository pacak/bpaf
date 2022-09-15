<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    banana: bool,
    switch: bool,
}

// accepts `-banana`, note a single dash
fn banana() -> impl Parser<bool> {
    short('b')
        .argument("anana")
        .adjacent()
        .guard(|b| b == "anana", "not anana")
        .optional()
        .catch()
        .map(|b| b.is_some())
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    // banana() is just a syntax construct! allows, not magic
    construct!(Options { banana(), switch }).to_options()
}
```

</details>
<details>
<summary>Examples</summary>


other than looking strange `banana()` should behave like a regular flag parser: banana - yes
```console
% app -banana -s
Options { banana: true, switch: true }
```

banana - no
```console
% app -s
Options { banana: false, switch: true }
```

this is also accepted but close enough I think
```console
% app -b=anana
Options { banana: true, switch: false }
```

</details>
