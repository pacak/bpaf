/// A way to represent xorg like flags, not a typical usage
use bpaf::*;
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
    extensions: Vec<(String, bool)>,
}

// matches literal name prefixed with - or +
fn toggle_options(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<bool> {
    any(meta, move |s: String| {
        if let Some(suf) = s.strip_prefix('+') {
            (suf == name).then_some(true)
        } else if let Some(suf) = s.strip_prefix('-') {
            (suf == name).then_some(false)
        } else {
            None
        }
    })
    .help(help)
    .anywhere()
}

// matches literal +ext and -ext followed by extension name
fn extension() -> impl Parser<(String, bool)> {
    let state = any("(+|-)ext", |s: String| match s.as_str() {
        "-ext" => Some(false),
        "+ext" => Some(true),
        _ => None,
    })
    .anywhere();

    let name = positional::<String>("EXT")
        .help("Extension to enable or disable, see documentation for the full list");
    construct!(state, name).adjacent().map(|(a, b)| (b, a))
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_options("(+|-)backing", "backing", "Set backing status").fallback(false);
    let xinerama =
        toggle_options("(+|-)xinerama", "xinerama", "Set Xinerama status").fallback(true);
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    let extensions = extension().many();
    construct!(Options {
        turbo,
        backing,
        xinerama,
        extensions,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
