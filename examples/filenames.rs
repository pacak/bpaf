//! This example shows how to use shell completion to ask for
//! a file with one of two extensions. If you want to specify just one
//! extension having it as something like "*.rs" is good enough

use bpaf::{positional, Parser, ShellComp};
use std::path::PathBuf;

fn main() {
    let parser = positional::<PathBuf>("FILE")
        .complete_shell(ShellComp::File {
            mask: Some("*.(md|toml)"),
        })
        .many()
        .to_options();

    let r = parser.run();
    println!("{:?}", r);
}
