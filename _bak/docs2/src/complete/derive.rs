//
use bpaf::*;
/// suggest completions for the input
fn completer(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let names = ["Yuri", "Lupusregina", "Solution", "Shizu", "Entoma"];
    names
        .iter()
        .filter(|name| name.starts_with(input))
        .map(|name| (*name, None))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, argument("NAME"), complete(completer))]
    /// Specify character's name
    name: String,
}
