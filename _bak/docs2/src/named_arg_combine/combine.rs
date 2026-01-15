//
use bpaf::*;
//
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub enum Output {
    ToFile(PathBuf),
    ToConsole,
}
pub fn options() -> OptionParser<(usize, Output, bool)> {
    // In most cases you don't keep `NamedArg` around long enough
    // to assign it a name
    let size = short('s')
        .long("size")
        .help("Maximum size to process")
        .argument("SIZE");

    // but it can be useful if you want to have several arguments
    // sharing exact set of names - for example a switch (req_flag)
    // and an argument;
    let output = short('o').long("output");

    let to_file = output
        .clone()
        .help("Save output to file")
        .argument("PATH")
        .map(Output::ToFile);
    let to_console = output
        .help("Print output to console")
        .req_flag(Output::ToConsole);

    // when combining multiple parsers that can conflict with each other
    // it's a good idea to put more general first:
    let output = construct!([to_file, to_console]);

    let verbose = short('v')
        .long("verbose")
        .long("detailed")
        .help("Produce a detailed report")
        .switch();

    construct!(size, output, verbose).to_options()
}
