#### Applying transformations to parsed values

Once field have a consumer you can start applying transformations from [`Parser`] trait.
Annotation share the same names and follow the same composition rules as in Combinatoric API.

```no_run
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
    let opts = options().run();
    println!("{:?}", opts);
}
```
