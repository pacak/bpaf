//
use bpaf::*;
#[derive(Debug, Clone)]
//
#[allow(dead_code)]
pub struct Options {
    turbo: bool,
    backing: bool,
    xinerama: bool,
}

fn toggle_option(name: &'static str, help: &'static str) -> impl Parser<bool> {
    any::<String, _, _>(name, move |s: String| {
        if let Some(rest) = s.strip_prefix('+') {
            (rest == name).then_some(true)
        } else if let Some(rest) = s.strip_prefix('-') {
            (rest == name).then_some(false)
        } else {
            None
        }
    })
    .help(help)
    .anywhere()
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_option("backing", "Backing status").fallback(false);
    let xinerama = toggle_option("xinerama", "Xinerama status").fallback(true);
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
