//! How to require presence of at least N values,
//!
//! like `val1 val2 ... valN ... valM`.

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
