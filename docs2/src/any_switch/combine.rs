//
use bpaf::{doc::*, *};
#[derive(Debug, Clone)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
}

fn toggle_option(name: &'static str, help: &'static str) -> impl Parser<bool> {
    // parse +name and -name into a bool
    any::<String, _, _>(name, move |s: String| {
        if let Some(rest) = s.strip_prefix('+') {
            (rest == name).then_some(true)
        } else if let Some(rest) = s.strip_prefix('-') {
            (rest == name).then_some(false)
        } else {
            None
        }
    })
    // set a custom usage and help metavariable
    .metavar(
        &[
            ("+", Style::Literal),
            (name, Style::Literal),
            (" | ", Style::Text),
            ("-", Style::Literal),
            (name, Style::Literal),
        ][..],
    )
    // set a custom help description
    .help(help)
    // apply this parser to all unconsumed items
    .anywhere()
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_option("backing", "Enable or disable backing")
        .fallback(false)
        .debug_fallback();
    let xinerama = toggle_option("xinerama", "enable or disable Xinerama")
        .fallback(true)
        .debug_fallback();
    let turbo = short('t')
        .long("turbo")
        .help("Engage the turbo mode")
        .switch();
    construct!(Options {
        turbo,
        backing,
        xinerama,
    })
    .to_options()
}
