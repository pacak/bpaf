use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    verbose: bool,
    file: Vec<String>,
}

fn options() -> OptionParser<Options> {
    let verbose = short('v').switch().help("Emit diagnostic messages");
    let file = positional("FILE").help("File to process").posix().many();
    construct!(Options { verbose, file }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
