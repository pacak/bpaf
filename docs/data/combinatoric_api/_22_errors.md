#### Types of failures

Let's consider a parser that takes an optional numeric argument

```rust,id:2
# use bpaf::*;
fn numeric() -> impl Parser<Option<usize>> {
    short('n')
        .argument::<usize>("N")
        .guard(|n| *n <= 10, "N must be at or below 10")
        .optional()
}

fn main() {
    println!("{:?}", numeric().run());
}
# pub fn options() -> OptionParser<Option<usize>> { numeric().to_options() }
```

```run,id:2
-n 1
```

```run,id:2

```

```run,id:2
-n 11
```

```run,id:2
-n five
```

`short('n').argument("N")` succeeds in first and third cases since the parameter is present and
it is a valid number, second one fails with "value not found", fourth one fails with "value is
not valid".

Result produced by `argument` gets handled by `guard`. Failures in cases 2 and 4
are passed as is, successes are checked with "less than 11" and turned into failures if check
fails - case 3.

Result of `guard` gets into `optional` which converts present values into `Some` values, "value
not found" errors into `None` and keeps the rest of the failures as is.
