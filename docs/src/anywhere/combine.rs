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

fn toggle_options(name: &'static str, help: &'static str) -> impl Parser<bool> {
    any::<String>(name)
        .help(help)
        .parse(move |s| {
            let (state, cur_name) = if let Some(rest) = s.strip_prefix('+') {
                (true, rest)
            } else if let Some(rest) = s.strip_prefix('-') {
                (false, rest)
            } else {
                return Err(format!("{} is not a toggle option", s));
            };
            if cur_name != name {
                Err(format!("{} is not a known toggle option name", cur_name))
            } else {
                Ok(state)
            }
        })
        .anywhere()
        .catch()
}

pub fn options() -> OptionParser<Options> {
    let backing = toggle_options("backing", "Backing status").fallback(false);
    let xinerama = toggle_options("xinerama", "Xinerama status").fallback(true);
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
