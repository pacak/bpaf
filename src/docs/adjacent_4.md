<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    multi: Vec<Multi>,
    switch: bool,
}

# #[allow(dead_code)]
#[derive(Debug, Clone)]
struct Multi {
    m: (),
    pos: usize,
    flag: bool,
    arg: Option<usize>,
}

/// You can mix all sorts of things inside the adjacent group
fn multi() -> impl Parser<Multi> {
    let m = short('m').req_flag(());
    let pos = positional::<usize>("POS");
    let arg = long("arg").argument::<usize>("ARG").optional();
    let flag = long("flag").switch();
    construct!(Multi { m, arg, flag, pos }).adjacent()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let multi = multi().many();
    construct!(Options { multi, switch }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Let's start simple - a single flag accepts a bunch of stuff, and eveything is present
```console
% app -m 330 --arg 10 --flag
Options { multi: [Multi { m: (), pos: 330, flag: true, arg: Some(10) }], switch: false }
```

You can omit some parts, but also have multiple groups thank to `many`
```console
% app -m 100 --flag    -m 30 --arg 10    -m 50
Options { multi: [Multi { m: (), pos: 100, flag: true, arg: None }, Multi { m: (), pos: 30, flag: false, arg: Some(10) }, Multi { m: (), pos: 50, flag: false, arg: None }], switch: false }
```

</details>
