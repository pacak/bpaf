//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    package: String,
}

fn package() -> impl Parser<String> {
    long("package")
        .short('p')
        .help("Package to use")
        .argument("SPEC")
        .adjacent()
}

pub fn options() -> OptionParser<Options> {
    construct!(Options { package() }).to_options()
}
