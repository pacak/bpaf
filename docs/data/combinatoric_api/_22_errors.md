#### Types of failures

Let's consider a parser that takes an optional numeric argument and makes sure it's below 10 if
present.

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

Option is present and is valid
```run,id:2
-n 1
```

Option is missing
```run,id:2

```

Option is present, it is a number but it's larger than the validation function allows
```run,id:2
-n 11
```

Option is present, but the value is not a number
```run,id:2
-n five
```

`short('n').argument("N")` part of the parser succeeds in the first and the third cases since
the parameter is present and it is a valid number, in the second care it fails with "value not
found", and in the fourth case it fails with "value is not valid".

Result produced by `argument` gets handled by `guard`. Failures in the second and the fourth
cases are passed as is, successes are checked with "less than 11" and turned into failures if
check fails - in the third case.

Result of `guard` gets into `optional` which converts present values into `Some` values, "value
not found" types of errors into `None` and keeps the rest of the failures as is.
