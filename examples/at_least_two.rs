//! How to require presence of at least N values,
//!
//! like `val1 val2 ... valN ... valM`.

use bpaf::*;

fn main() {
    let flag = short('f')
        .req_switch()
        .many()
        .guard(|x| x.len() >= 2, "at least two arguments are required");

    let opt = run(Info::default().for_parser(flag));
    println!("{:?}", opt);
}
