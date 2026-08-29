use bpaf::*;

/// Custom value with `FromStr` instance
#[derive(Debug, Clone)]
struct Rts;

impl std::str::FromStr for Rts {
    // the error type is not important, any_from_str ignores the error value
    type Err = std::fmt::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+RTS" => Ok(Rts),
            _ => Err(std::fmt::Error),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    item: i32,
    rts: Rts,
}

fn options() -> OptionParser<Options> {
    let item = any_from_str::<i32>("NUM").help("Help for the first item");
    let rts = any_from_str::<Rts>("RTS").help("Enable RTS mode with '+RTS'");
    construct!(Options { item, rts }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
