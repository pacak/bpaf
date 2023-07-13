//! Simple dynamic completion example

#![allow(dead_code)]
use bpaf::*;

fn crates(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
    let crates = [
        (
            "cargo-hackerman",
            "Workspace hack management and package/feature query",
        ),
        ("cargo-prebuilt", "Download prebuilt crate binaries"),
        ("cargo-show-asm", "Display generated assembly"),
        (
            "cargo-supply-chain",
            "Gather author, contributor, publisher data on crates",
        ),
        ("chezmoi_modify_manager", "Chezmoi addon to patch ini files"),
        ("xvf", "Easy archive extraction"),
        ("newdoc", "Generate pre-populated module files"),
        (
            "nust64",
            "Tools for compiling a Rust project into an N64 ROM",
        ),
        ("uggo", "CLI tool to query builds from u.gg"),
    ];

    crates
        .iter()
        .filter(|p| p.0.starts_with(input))
        .map(|name| (name.0, Some(name.1)))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, Copy, Bpaf)]
/// Format for generated report
#[bpaf(fallback(Format::Text))]
enum Format {
    /// Generate report in JSON format
    Json,
    /// Generate report in XML format
    Xml,
    /// Generate report in plaintext format
    Text,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Select crate for analysis
    #[bpaf(long("crate"), argument("NAME"), complete(crates))]
    name: String,
    /// Include dependencies into report
    dependencies: bool,
    #[bpaf(external)]
    format: Format,
    /// Upload report to a url
    #[bpaf(positional("URL"))]
    upload: Option<String>,
}

fn main() {
    println!("{:?}", options().run());
}
