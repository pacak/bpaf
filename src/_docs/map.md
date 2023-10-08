## Derive example

`map` takes a single parameter. It can be name of a function name in scope or a closure

````rust
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument::<u32>("N"), map(|x| x * 2))]
    number: u32,
}
````

## Combinatoric example

````rust
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    number: u32,
}

pub fn options() -> OptionParser<Options> {
    let number = long("number").argument::<u32>("N").map(|x| x * 2);
    construct!(Options { number }).to_options()
}
````

You can use `map` to apply arbitrary pure transformation to any input.
Here `--number` takes a numerical value and doubles it



```text
$ app --number 10
Options { number: 20 }
```


But if function inside the parser fails - user will get the error back unless it's handled
in some way.



```text
$ app --number ten
Error: couldn't parse `ten`: invalid digit found in string
```

