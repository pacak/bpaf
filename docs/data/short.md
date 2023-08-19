Parse an [`argument`](NamedArg::argument), a [`switch`](NamedArg::switch) or a [`flag`](NamedArg::flag) that has a short name

#### Combinatoric usage

Once called `short` gives a [`NamedArg`](parsers::NamedArg) object which implements [`short`](NamedArg::short)
method too so you can add multiple short names to your parsers. First short name stays visible
in the help message and documentation, the rest become hidden aliases.

To turn that into a parser you might want to attach a [`help`](NamedArg::help) message and finally
convert it to a [`Parser`](crate::Parser) using an [`argument`](NamedArg::argument), a [`switch`](NamedArg::switch),
a [`flag`](NamedArg::flag) or a [`req_flag`](NamedArg::req_flag) methods.


```rust,id:1
# use bpaf::*;
fn parser() -> impl Parser<bool> {
    short('s')      // visible name
        .short('S') // hidden alias
        .help("A custom switch with a short name")
        .switch()
}
# pub fn options() -> OptionParser<bool> { parser().to_options() }
```

Help message contains only the visible name

```run,id:1
--help
```

But parser accepts both `-s` and `-S`

```run,id:1
-s
```
```run,id:1
-S
```

#### Derive usage

For derive macro `short` annotation goes either on a field that belongs to `struct` or `enum`
variant or directly on enum variant itself.

```rust,id:2,fold:"Combinatoric example"
use bpaf::Bpaf;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# pub
struct Options {
    /// A custom switch with a short name
    #[bpaf(short, short('S'))]
    switch: bool,
}
```

Help message contains only the visible name

```run,id:2
--help
```

But parser accepts both `-s` and `-S`

```run,id:2
-s
```

```run,id:2,fold:"Hidden alias"
-S
```

Usage on a enum variant with no fields:

```rust,id:3
use bpaf::Bpaf;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# pub
enum Options {
    /// A variant Alpha that does something
    #[bpaf(short)]
    Alpha,
    /// A variant Beta that does something else
    #[bpaf(short('B'))]
    Beta,
}
```

```run,id:3
--help
```

```run,id:3
-a
```

