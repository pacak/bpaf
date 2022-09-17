//! You can open files as part of parsing process too, might not be the best idea though
//! because depending on a context `bpaf` might need to evaluate some parsers multiple times

use bpaf::*;
use std::{
    ffi::OsString,
    fs::File,
    io::{stdin, BufRead, BufReader, Read},
};

fn main() {
    let file = positional::<OsString>("FILE")
        .parse::<_, Box<dyn Read>, std::io::Error>(|path| {
            Ok(if path == "-" {
                Box::new(stdin())
            } else {
                Box::new(File::open(path)?)
            })
        })
        .to_options()
        .run();

    let reader = BufReader::new(file);

    for line in reader.lines() {
        println!("{:?}", line.unwrap());
    }
}
