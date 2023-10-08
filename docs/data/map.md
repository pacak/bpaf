## Derive example
`map` takes a single parameter. It can be name of a function name in scope or a closure

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<u32>("N"), map(|x| x * 2))]
    number: u32,
}
```

## Combinatoric example
```rust,id:2
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").map(|x| x * 2);
    construct!(Options { number }).to_options()
}
```


You can use `map` to apply arbitrary pure transformation to any input.
Here `--number` takes a numerical value and doubles it

```run,id:1,id:2
--number 10
```

But if function inside the parser fails - user will get the error back unless it's handled
in some way.

```run,id:1,id:2
--number ten
```
