//
use bpaf::*;
#[derive(Debug, Clone)]
pub enum Style {
    Intel,
    Att,
    Llvm,
}

#[derive(Debug, Clone)]
pub enum Report {
    /// Include defailed report
    Detailed,
    /// Include minimal report
    Minimal,
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone)]
pub struct Options {
    agree: (),
    style: Style,
    report: Report,
}

pub fn options() -> OptionParser<Options> {
    let agree = long("agree")
        .help("You must agree to perform the action")
        .req_flag(());

    let intel = long("intel")
        .help("Show assembly using Intel style")
        .req_flag(Style::Intel);
    let att = long("att")
        .help("Show assembly using AT&T style")
        .req_flag(Style::Att);
    let llvm = long("llvm").help("Show llvm-ir").req_flag(Style::Llvm);
    let style = construct!([intel, att, llvm]);

    let detailed = long("detailed")
        .help("Include detailed report")
        .req_flag(Report::Detailed);
    let minimal = long("minimal")
        .help("Include minimal report")
        .req_flag(Report::Minimal);
    let report = construct!([detailed, minimal]).fallback(Report::Undecided);

    construct!(Options {
        agree,
        style,
        report
    })
    .to_options()
}
