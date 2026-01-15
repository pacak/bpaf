//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    release: bool,
    binary: String,
}

pub fn options() -> OptionParser<Options> {
    let release = short('r')
        .long("release")
        .help("Perform actions in release mode")
        .switch();

    let binary = short('b')
        .long("binary")
        .help("Use this binary")
        .argument("BIN");

    construct!(Options { release, binary })
        .to_options()
        .with_usage(|u| {
            let mut doc = Doc::default();
            doc.emphasis("Usage: ");
            doc.literal("my_program");
            doc.text(" ");
            doc.doc(&u);
            doc
        })
}
