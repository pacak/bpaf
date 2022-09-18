//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
//
#[allow(dead_code)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long, short('S'), long("also-switch"))]
    /// Switch with many names
    switch: bool,
    #[bpaf(short, long("argument"), short('A'), long("also-arg"))]
    /// Argument with names
    arg: usize,
    #[bpaf(short, long("user"), env("USER1"), argument("USER"))]
    /// Custom user name
    username: String,
}
