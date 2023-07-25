#### Numeric flags - compression levels like in zip

While you can add flags in a usual way for compression levels using `short(1)`, `short(2)`, etc
combined with `req_flag`, you can also parse all of then using [`any`]

```rust
use bpaf::{doc::Style, *};

fn compression() -> impl Parser<usize> {
    any::<isize, _, _>("COMP", |x: isize| {
        if (-9..=-1).contains(&x) {
            Some(x.abs().try_into().unwrap())
        } else {
            None
        }
    })
    .metavar(&[
        ("-1", Style::Literal),
        (" to ", Style::Text),
        ("-9", Style::Literal),
    ])
    .help("Compression level")
    .anywhere()
}

fn main() {
    let opts = compression().to_options().run();

    println!("{:?}", opts);
}
```
