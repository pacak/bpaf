#### Transforming parsed values

Once the field has a consumer you can start applying transformations from the [`Parser`] trait.
Annotation share the same names and follow the same composition rules as in Combinatoric API.


```rust,id:1
# use bpaf::*;
fn small(size: &usize) -> bool {
    *size < 10
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // double the width
    #[bpaf(short, argument::<usize>("PX"), map(|w| w*2))]
    width: usize,

    // make sure the hight is below 10
    #[bpaf(argument::<usize>("LENGTH"), guard(small, "must be less than 10"))]
    height: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

Help as usual

```run,id:1
--help
```

And parsed values are differnt from what user passes

```run,id:1
--width 10 --height 3
```

Additionally height cannot exceed 10

```run,id:1
--width 555 --height 42
```
