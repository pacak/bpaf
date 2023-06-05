//
use bpaf::{doc::*, *};
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Engage the turbo mode
    #[bpaf(short, long)]
    turbo: bool,
    #[bpaf(external(backing), fallback(false), debug_fallback)]
    backing: bool,
    #[bpaf(external(xinerama), fallback(true), debug_fallback)]
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

fn backing() -> impl Parser<bool> {
    toggle_option("backing", "Enable or disable backing")
}

fn xinerama() -> impl Parser<bool> {
    toggle_option("xinerama", "enable or disable Xinerama")
}
