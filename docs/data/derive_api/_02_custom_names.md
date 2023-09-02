#### Customizing flag and argument names

By default names for flags are taken directly from the field names so usually you don't
have to do anything about it, but you can change it with annotations on the fields themselves.

Rules for picking the name are:

1. With no annotations field name longer than a single character becomes a long name,
   single character name becomes a short name
2. Adding either `long` or `short` disables item 1, so adding `short` disables the long name
3. `long` or `short` annotation without a parameter derives a value from a field name
4. `long` or `short` with a parameter uses that instead
5. You can have multiple `long` and `short` annotations, the first of each type becomes a
   visible name, remaining are used as hidden aliases

And if you decide to add names - they should go to the left side of the annotation list.


```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A custom switch
    #[bpaf(short, long)]
    switch: bool,

    /// A custom argument
    #[bpaf(long("my-argument"), short('A'))]
    argument: usize,
}

fn main() {
    println!("{:?}", options().run());
}
```

`bpaf` uses custom names in help message

```run,id:1
--help
```

As well as accepts them on a command line and uses in error message

```run,id:1
--switch
```

```run,id:1
-A 42 -s
```
