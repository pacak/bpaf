//
use bpaf::{doc::*, *};

const ARG: &[(&str, Style)] = &[
    ("Very", Style::Emphasis),
    (" important argument", Style::Text),
];

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(argument("N"), help(ARG))]
    number: u32,
}
