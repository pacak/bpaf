//! You can open files as part of parsing process too, might not be the best idea though
//! because depending on a context `bpaf` might need to evaluate some parsers multiple times.
//!
//! Main motivation for this example is that you can open a file as part of the argument parsing
//! and give a reader directly to user. In practice to replicate `cat`'s behavior you'd accept
//! multiple files with `many` and open them one by one in your code.

use bpaf::*;
use std::{
    ffi::OsString,
    fs::File,
    io::{stdin, BufRead, BufReader, Read},
};

fn main() {
    let file = positional::<OsString>("FILE")
        .help("File name to concatenate, with no FILE or when FILE is -, read standard input")
        .optional()
        .parse::<_, Box<dyn Read>, std::io::Error>(|path| {
            Ok(if let Some(path) = path {
                if path == "-" {
                    Box::new(stdin())
                } else {
                    Box::new(File::open(path)?)
                }
            } else {
                Box::new(stdin())
            })
        })
        .to_options()
        .descr("Concatenate a file to standard output")
        .run();

    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{}", line.unwrap());
    }
}
