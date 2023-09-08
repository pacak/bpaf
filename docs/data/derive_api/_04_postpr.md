#### Transforming parsed values

Often specifying consumer is enough to parse a value, but in some cases you might want to apply
additional transformations or validations. for example some numeric parameter must be not only
valid `u8`, but also in range 1..100 inclusive or an IP address should belong to a certain
range. On the right side of the consumer you can list zero or more transformations from the
[`Parser`] trait. Annotation share the same names and follow the same composition rules as in
Combinatoric API.


```rust,id:1
use bpaf::*;
fn small(size: &usize) -> bool {
    *size < 10
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // double the width
    #[bpaf(argument::<usize>("PX"), map(|w| w*2))]
    width: usize,

    // make sure the hight is below 10
    #[bpaf(argument::<usize>("LENGTH"), guard(small, "must be less than 10"))]
    height: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

And parsed values are differnt from what user passes

```run,id:1
--width 10 --height 3
```

Additionally height cannot exceed 10

```run,id:1
--width 555 --height 42
```
