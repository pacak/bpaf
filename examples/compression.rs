//! Numeric flags similar to zip compression levels, accepts -1 to -9, produces usize

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
