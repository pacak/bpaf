#### Customizing flag and argument names

By default names for flags are taken directly from the field names so usually you don't
have to do anything about it, but you can change it with annotations on the fields themselves.

Rules for picking the name are:

1. With no annotations field name longer than a single character becomes a long name,
   single character name becomes a short name:

```rust,id:1
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// A switch with a long name
    switch: bool,
    /// A switch with a short name
    a: bool,
}

fn main() {
    println!("{:?}", options().run());
}
```

In this example `switch` and `a` are implicit long and short names, help message lists them

```run,id:1
--help
```

2. Adding either `long` or `short` disables rule 1, so adding `short` disables the long name

```rust,id:2
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short)]
    /// A switch with a long name
    switch: bool,

    #[bpaf(long)]
    /// A switch with a short name
    s: bool,
}

fn main() {
    println!("{:?}", options().run());
}
```

Here implicit names are replaced with explicit ones, derived from field names. `--s` is a
strange looking long name, but that's what's available

```run,id:2
--help
```

3. `long` or `short` with a parameter uses that value instead

```rust,id:3
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short('S'))]
    /// A switch with a long name
    switch: bool,

    #[bpaf(long("silent"))]
    /// A switch with a short name
    s: bool,
}

fn main() {
    println!("{:?}", options().run());
}
```

Here names are `-S` and `--silent`, old names are not available

```run,id:3
--help
```

4. You can have multiple `long` and `short` annotations, the first of each type becomes a
   visible name, remaining are used as hidden aliases


```rust,id:4
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short('v'), short('H'))]
    /// A switch with a long name
    switch: bool,

    #[bpaf(long("visible"), long("hidden"))]
    /// A switch with a short name
    s: bool,
}

fn main() {
    println!("{:?}", options().run());
}
```

Here parser accepts 4 different names, visible `-v` and `--visible` and two hidden aliases:
`-H` and `--hidden`

```run,id:4
--help
```

```run,id:4
-v --visible
```

Aliases don't show up in the help message or anywhere else but still work.

```run,id:4
-H --hidden
```

And if you decide to add names - they should go to the left side of the annotation list.
