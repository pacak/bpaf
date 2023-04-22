<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    multi: Vec<Rect>,
}

# #[allow(dead_code)]
#[derive(Debug, Clone)]
struct Rect {
    item: (),
    width: usize,
    height: usize,
    painted: bool,
}

fn multi() -> impl Parser<Rect> {
    let item = long("rect").req_flag(());
    let width = long("width").argument::<usize>("PX");
    let height = long("height").argument::<usize>("PX");
    let painted = long("painted").switch();
    construct!(Rect {
        item,
        width,
        height,
        painted,
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
<summary style="display: list-item;">Examples</summary>


Order of items within the rectangle is not significant and you can have several of them
```console
% app --rect --width 10 --height 10 --rect --height 10 --width 10
Options { switch: false, multi: [Rect { item: (), width: 10, height: 10, painted: false }, Rect { item: (), width: 10, height: 10, painted: false }] }
```

You can have optional values that belong to the group inside and outer flags in the middle
```console
% app --rect --width 10 --painted --height 10 -s --rect --height 10 --width 10
Options { switch: true, multi: [Rect { item: (), width: 10, height: 10, painted: true }, Rect { item: (), width: 10, height: 10, painted: false }] }
```

But with `adjacent` they cannot interleave
```console
% app --rect --rect --width 10 --painted --height 10 --height 10 --width 10
Expected --width <PX>, got "--rect". Pass --help for usage information
```

</details>
