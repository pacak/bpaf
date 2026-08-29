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

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(any_from_str("NUM"))]
    /// Help for the first item
    item: i32,
    #[bpaf(any_from_str("RTS"))]
    /// Enable RTS mode with '+RTS'
    rts: Rts,
}

fn main() {
    println!("{:?}", options().run());
}
