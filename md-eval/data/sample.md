This is an example


```fold,comb
# use bpaf::*;
pub struct Options {
    a: bool,
}

fn options() -> OptionParser<Options> {
    let a = short('a').switch();
    construct!(Options { a }).to_options()
}
```

```fold,der
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    a: bool,
}
```



You can run it like this
```shell
--help
```

```shell
-a
```
