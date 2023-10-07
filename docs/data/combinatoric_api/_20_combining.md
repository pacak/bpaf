#### Combining multiple simple parsers

A single-item option parser can only get you so far. Fortunately, you can combine multiple
parsers with [`construct!`] macro.

For sequential composition (all the fields must be present) you write your code as if you are
constructing a structure, enum variant or a tuple and wrap it with `construct!`. Both
a constructor and parsers must be present in the scope. If instead of a parser you have a function
that creates one - just add `()` after the name:

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Copy)]
# pub
struct Options {
    alpha: usize,
    beta: usize
}

fn alpha() -> impl Parser<usize> {
    long("alpha").argument("ALPHA")
}

fn both() -> impl Parser<Options> {
    let beta = long("beta").argument("BETA");
    // call `alpha` function, and use result to make parser
    // for field `alpha`,
    // use parser `beta` from the scope for field `beta`
    construct!(Options { alpha(), beta })
}

fn main() {
    println!("{:?}", both().run());
}
# pub fn options() -> OptionParser<Options> { both().to_options() }
```

```run,id:1
--alpha 10 --beta 20
```

For named parsers order doesn't matter

```run,id:1
--beta 20 --alpha 10
```

```run,id:1
--help
```

If you are using positional parsers - they must go to the right-most side and will run in
the order you specify them. For named parsers order affects only the `--help` message.

The second type of composition `construct!` offers is a parallel composition. You pass multiple
parsers that produce the same result type in `[]` and `bpaf` selects one that fits best with
the data user gave.


```rust,id:2
# use bpaf::*;
fn distance() -> impl Parser<f64> {
    let km = long("km").help("Distance in km").argument::<f64>("KM");
    let miles = long("mi").help("Distance in miles").argument::<f64>("MI").map(|d| d * 1.621);
    construct!([km, miles])
}

fn main() {
    println!("{:?}", distance().run());
}
# pub fn options() -> OptionParser<f64> { distance().to_options() }
```

Parser `distance` accepts either `--km` or `--mi`, but not both at once and produces a single `f64` converted to km.
```run,id:2
--km 42
```

```run,id:2
--mi 42
```

```run,id:2
--km 42 --mi 42
```

Help indicates that either value is accepted

```run,id:2
--help
```


If parsers inside parallel composition parse the same items from the command line - the longest
possible match should go first since `bpaf` picks an earlier parser if everything else is
equal, otherwise it does not matter. In this example `construct!([miles, km])` produces the
same results as `construct!([km, miles])` and only `--help` message is going to be different.

Parsers created with [`construct!`] still implement the [`Parser`] trait so you can apply more
transformation on top. For example same as you can make a simple parser optional - you can make
a composite parser optional. Parser transformed this way will succeed if both `--alpha` and
`--beta` are present or neither of them:

```rust,id:3
# use bpaf::*;
# #[derive(Debug, Clone)] pub
struct Options {
    alpha: usize,
    beta: usize
}

fn parser() -> impl Parser<Option<Options>> {
    let alpha = long("alpha").argument("ALPHA");
    let beta = long("beta").argument("BETA");
    construct!(Options { alpha, beta }).optional()
}

fn main() {
    println!("{:?}", parser().run() );
}
# pub fn options() -> OptionParser<Option<Options>> { parser().to_options() }
```

```run,id:3
--help
```

Here `optional` parser returns `Some` value if inner parser succeeds

```run,id:3
--alpha 10 --beta 15
```

Or `None` if neither value is present

```run,id:3

```

For parsers that are partially successfull user will get an error

```run,id:3
--alpha 10
```
