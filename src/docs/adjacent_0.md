<details>
<summary>Combinatoric usage</summary>

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
    val_1: usize,
    val_2: usize,
    val_3: f64,
}

fn multi() -> impl Parser<Multi> {
    let m = short('m').req_flag(());
    let val_1 = positional::<usize>("V1");
    let val_2 = positional::<usize>("V2");
    let val_3 = positional::<f64>("V3");
    construct!(Multi {
        m,
        val_1,
        val_2,
        val_3
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let multi = multi().many();
    construct!(Options { multi, switch }).to_options()
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
    #[bpaf(external, many)]
    multi: Vec<Multi>,
    #[bpaf(short)]
    switch: bool,
}

# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Multi {
    m: (),
    #[bpaf(positional("V1"))]
    val_1: usize,
    #[bpaf(positional("V2"))]
    val_2: usize,
    #[bpaf(positional("V3"))]
    val_3: f64,
}
```

</details>
<details>
<summary>Examples</summary>


short flag `-m` takes 3 positional arguments: two integers and one floating point, order is
important, switch `-s` can go on either side of it
```console
% app -s -m 10 20 3.1415
Options { multi: [Multi { m: (), val_1: 10, val_2: 20, val_3: 3.1415 }], switch: true }
```

parser accepts multiple groups of `-m` - they must not interleave
```console
% app -s -m 10 20 3.1415 -m 1 2 0.0
Options { multi: [Multi { m: (), val_1: 10, val_2: 20, val_3: 3.1415 }, Multi { m: (), val_1: 1, val_2: 2, val_3: 0.0 }], switch: true }
```

`-s` can't go in the middle as the parser expects the second item
```console
% app -m 10 20 -s 3.1415
Expected <V3>, pass --help for usage information
```

</details>
