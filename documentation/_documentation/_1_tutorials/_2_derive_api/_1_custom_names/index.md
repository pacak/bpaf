#### Customizing flag and argument names

By default names for flag names are taken directly from the field names so usually you don't
have to do anything about it, but you can change it with annotations on the fields themselves:

```no_run
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
    let opts = options().run();
    println!("{:?}", opts);
}
```

Rules for picking the name are:

1. With no annotations field name longer than a single character becomes a long name,
   single character name becomes a short name
2. Adding either `long` or `short` disables item 1, so adding `short` disables long name
3. `long` or `short` annotation without a parameter derives a value from a field name
4. `long` or `short` with a parameter uses that instead
5. You can have multiples `long` and `short` annotations, first of each type becomes a
   visible name, remaining are used as hidden aliases

And if you decide to add names - they should go to the left side of the annotation list
