#### Transforming parsers

Once you have your simple parsers implemented you might want to improve them further - add
fallback values, or change them to consume multiple items, etc. Every primitive (or composite)
parser implements [`Parser`] so most of the transformations are coming from this trait.

Say you have a parser that takes a crate name as a required argument:

```rust
use bpaf::*;
fn krate() -> impl Parser<String> {
    long("crate").help("Crate name to process").argument("CRATE")
}
```

You can turn it into, for example, an optional argument - something that returns
`Some("my_crate")` if specified or `None` if it wasn't. Or to let the user to pass a multiple
of them and collect them all into a `Vec`


```rust,id:1
# use bpaf::*;
fn maybe_krate() -> impl Parser<Option<String>> {
    long("crate")
        .help("Crate name to process")
        .argument("CRATE")
        .optional()
}

fn many_krates() -> impl Parser<Vec<String>> {
    long("crate")
        .help("Crate name to process")
        .argument("CRATE")
        .many()
}

fn main() {
    println!("{:?}", many_krates().run());
}
# pub fn options() -> OptionParser<Vec<String>> { many_krates().to_options() }
```

```run,id:1
--crate bpaf --crate luhn3
```

Transforming a parser with a method from the `Parser` trait usually gives you a new parser back and
you can chain as many transformations as you need.

Transformations available in the `Parser` trait are things like adding fallback values, making
the parser optional, making it so it consumes many but at least one value, changing how it is
being shown in `--help` output, adding additional validation and parsing on top and so on.

The order of those chained transformations matters and for some operations using the right order
makes code cleaner. For example, suppose you are trying to write a parser that takes an even
number and this parser should be optional. There are two ways to write it:

Validation first:

```rust,id:2
# use bpaf::*;
fn even() -> impl Parser<Option<usize>> {
    long("even")
        .argument("N")
        .guard(|&n| n % 2 == 0, "number must be even")
        .optional()
}
# pub fn options() -> OptionParser<Option<usize>> { even().to_options() }
```

Optional first:

```rust
# use bpaf::*;
fn even() -> impl Parser<Option<usize>> {
    long("even")
        .argument("N")
        .optional()
        .guard(|&n| n.map_or(true, |n| n % 2 == 0), "number must be even")
}
```

In later case validation function must deal with a possibility where a number is absent, for this
specific example it makes code less readable.

Result is identical in both cases:

```run,id:2
--even 2
```

```run,id:2
--even 3
```

```run,id:2

```

One of the important types of transformations you can apply is a set of failing
transformations. Suppose your application operates with numbers and uses `newtype` pattern to
keep track of what numbers are odd or even. A parser that consumes an even number can use
[`Parser::parse`] and may look like this:

```rust,id:3
# use bpaf::*;
#[derive(Debug, Clone, Copy)]
pub struct Even(usize);

fn mk_even(n: usize) -> Result<Even, &'static str> {
    if n % 2 == 0 {
        Ok(Even(n))
    } else {
        Err("Not an even number")
    }
}

fn even() -> impl Parser<Even> {
    long("even")
        .argument::<usize>("N")
        .parse(mk_even)
}

fn main() {
    println!("{:?}", even().run());
}
# pub fn options() -> OptionParser<Even> { even().to_options() }
```

User gets the same/similar output, but the application gets a value in a `newtype` wrapper.

```run,id:3
--even 2
```

```run,id:3
--even 3
```

```run,id:3

```
