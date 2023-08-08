Parse an [`argument`](NamedArg::argument), a [`switch`](NamedArg::switch) or a [`flag`](NamedArg::flag) that has a short name

#### Combinatoric usage

Once called `short` gives a [`NamedArg`](parsers::NamedArg) object which implements [`short`](NamedArg::short)
method too so you can add multiple short names to your parsers. First short name stays visible
in the help message and documentation, the rest become hidden aliases.

To turn that into a parser you might want to attach a [`help`](NamedArg::help) message and finally
convert it to a [`Parser`](crate::Parser) using an [`argument`](NamedArg::argument), a [`switch`](NamedArg::switch),
a [`flag`](NamedArg::flag) or a [`req_flag`](NamedArg::req_flag) methods.

````rust
# use bpaf::*;
fn parser() -> impl Parser<bool> {
    short('s')      // visible name
        .short('S') // hidden alias
        .help("A custom switch with a short name")
        .switch()
}
# pub fn options() -> OptionParser<bool> { parser().to_options() }
````

Help message contains only the visible name



```text
$ app --help
Usage: app [-s]

Available options:
    -s          A custom switch with a short name
    -h, --help  Prints help information
```


But parser accepts both `-s` and `-S`



```text
$ app -s
true
```



```text
$ app -S
true
```


#### Derive usage

For derive macro `short` annotation goes either on a field that belongs to `struct` or `enum`
variant or directly on enum variant itself.

<details><summary>"Combinatoric example"</summary>

````rust
use bpaf::Bpaf;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# pub
struct Options {
    /// A custom switch with a short name
    #[bpaf(short, short('S'))]
    switch: bool,
}
````

</details>
Help message contains only the visible name



```text
$ app --help
Usage: app [-s]

Available options:
    -s          A custom switch with a short name
    -h, --help  Prints help information
```


But parser accepts both `-s` and `-S`



```text
$ app -s
Options { switch: true }
```

<details><summary>"Hidden alias"</summary>

```text
$ app -S
Options { switch: true }
```

</details>
Usage on a enum variant with no fields:

````rust
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
````



```text
$ app --help
Usage: app (-a | -B)

Available options:
    -a          A variant Alpha that does something
    -B          A variant Beta that does something else
    -h, --help  Prints help information
```



```text
$ app -a
Alpha
```

