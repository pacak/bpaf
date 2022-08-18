//! How to require presence of at least N values,
//!
//! This program accepts "-f -f -f" or "-fffff" but not "-f"

use bpaf::*;

fn main() {
    let opt = short('f')
        .req_flag(())
        .many()
        .guard(|x| x.len() >= 2, "at least two arguments are required")
        .to_options()
        .run();

    println!("{:?}", opt);
}
