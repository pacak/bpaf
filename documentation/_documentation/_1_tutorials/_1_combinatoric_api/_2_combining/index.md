#### Combining multiple simple parsers

A single item option parser can only get you so far. Fortunately you can combine multiple
parsers together with [`construct!`] macro.

For sequential composition (all the fields must be present) you write your code as if you are
constructing a structure, enum variant or a tuple and wrap it with `construct!`. Both
constructor and parsers must be present in scope. If instead of a parser you have a function
that creates one - just add `()` after the name:

```rust
# use bpaf::*;
struct Options {
    alpha: usize,
    beta: usize
}

fn alpha() -> impl Parser<usize> {
    long("alpha").argument("ALPHA")
}

fn both() -> impl Parser<Options> {
    let beta = long("beta").argument("BETA");
    construct!(Options { alpha(), beta })
}
```

Full example:
#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_construct.md"))]

If you are using positional parsers - they must go to the right most side and will run in
order you specify them. For named parsers order affects only the `--help` message.

Second type of composition `construct!` offers is a parallel composition. You pass multiple
parsers that produce the same result type and `bpaf` runs one that fits best with the data user
gave.


#![cfg_attr(not(doctest), doc = include_str!("docs2/compose_basic_choice.md"))]

If parsers inside parallel composition can parse the same object - longest possible match
should go first since `bpaf` picks earlier parser if everything else is equal, otherwise it
does not matter. In this example `construct!([miles, km])` produces the same results as
`construct!([km, miles])` and only `--help` message is going to be different.

Parsers created with [`construct!`] still implement [`Parser`] trait so you can apply more
transformation on top. For example same as you can make a simple parser optional - you can make
composite parser optional. Such parser will succeed if both `--alpha` and `--beta` are
present or neither of them:

```rust
# use bpaf::*;
struct Options {
    alpha: usize,
    beta: usize
}

fn parser() -> impl Parser<Option<Options>> {
    let alpha = long("alpha").argument("ALPHA");
    let beta = long("beta").argument("BETA");
    construct!(Options { alpha, beta }).optional()
}
```
