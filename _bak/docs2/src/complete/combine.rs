//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    name: String,
}

fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    names
        .iter()
        .filter(|name| name.starts_with(input))
        .map(|name| (*name, None))
        .collect::<Vec<_>>()
}

pub fn options() -> OptionParser<Options> {
    let name = short('n')
        .long("name")
        .help("Specify character's name")
        .argument("NAME")
        .complete(completer);
    construct!(Options { name }).to_options()
}
