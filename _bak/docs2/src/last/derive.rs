//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(last)]
pub enum Style {
    /// Show assembly using Intel style
    Intel,
    /// Show assembly using AT&T style
    Att,
    /// Show llvm-ir
    Llvm,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(last, fallback(Report::Undecided))]
pub enum Report {
    /// Include detailed report
    Detailed,
    /// Include minimal report
    Minimal,
    #[bpaf(skip)]
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // external here uses explicit reference to function `style`
    // generated above
    #[bpaf(external(style))]
    style: Style,
    // here reference is implicit and derived from field name: `report`
    #[bpaf(external)]
    report: Report,
}
