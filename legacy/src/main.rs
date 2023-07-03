//! Parsing snippet from cargo-show-asm
//! Derive + typed fallback + external both with and without name

use bpaf::{construct, long, Bpaf, Parser, ShellComp};
use std::{convert::Infallible, path::PathBuf};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options("asm"))] // derives cargo helper for cargo-asm
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    #[bpaf(external(parse_manifest_path))]
    pub manifest_path: PathBuf,
    /// Custom target directory for generated artifacts
    #[bpaf(argument("DIR"))]
    pub target_dir: Option<PathBuf>,
    /// Package to use if ambigous
    #[bpaf(long, short, argument("SPEC"))]
    pub package: Option<String>,
    #[bpaf(external, optional)]
    pub focus: Option<Focus>,
    /// Produce a build plan instead of actually building
    pub dry: bool,
    /// Requires Cargo.lock and cache are up to date
    pub frozen: bool,
    /// Requires Cargo.lock is up to date
    pub locked: bool,
    /// Run without accessing the network
    pub offline: bool,
    #[bpaf(external)]
    pub format: Format,
    #[bpaf(external, fallback(Syntax::Intel))]
    pub syntax: Syntax,
    #[bpaf(external)]
    pub selected_function: SelectedFunction,
}

#[derive(Debug, Clone, Bpaf)]
/// Item to pick from the output
pub struct SelectedFunction {
    /// Complete or partial function name to filter
    #[bpaf(positional("FUNCTION"))]
    pub function: Option<String>,
    /// Select nth item from a filtered list
    #[bpaf(positional("INDEX"), fallback(0))]
    pub nth: usize,
}

fn parse_manifest_path() -> impl Parser<PathBuf> {
    long("manifest-path")
        .help("Path to Cargo.toml")
        .argument::<PathBuf>("PATH")
        .complete_shell(ShellComp::File {
            mask: Some("*.toml"),
        })
        .parse(|p| {
            // cargo-metadata wants to see
            if p.is_absolute() {
                Ok(p)
            } else {
                std::env::current_dir()
                    .map(|d| d.join(p))
                    .and_then(|full_path| full_path.canonicalize())
            }
        })
        .fallback_with(|| std::env::current_dir().map(|x| x.join("Cargo.toml")))
}

#[derive(Debug, Clone, Bpaf)]
/// How to render output
pub struct Format {
    /// Print interleaved Rust code
    pub rust: bool,

    #[bpaf(external(color_detection))]
    pub color: bool,

    /// include full demangled name instead of just prefix
    pub full_name: bool,
}

#[derive(Debug, Clone, Bpaf)]
/// Pick output type
///
/// included help
///
///
/// Extended help
pub enum Syntax {
    /// Generate assembly using Intel style
    Intel,
    /// Generate assembly using AT&T style
    Att,
}

fn color_detection() -> impl Parser<bool> {
    let yes = long("color")
        .help("Enable color highlighting")
        .req_flag(true);
    let no = long("no-color")
        .help("Disable color highlighting")
        .req_flag(false);
    construct!([yes, no]).fallback_with::<_, Infallible>(|| {
        // we can call for supports-color crate here
        Ok(true)
    })
}

fn comp_examples(prefix: &String) -> Vec<(String, Option<String>)> {
    // in the actual app we can ask cargo-metadata for this info
    let examples = ["derive_show_asm", "coreutils", "comonad"];
    examples
        .iter()
        .filter_map(|e| {
            if e.starts_with(prefix) {
                Some((e.to_string(), None))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone, Bpaf)]
/// Select artifact to use for analysis
///
/// Only one is valid
pub enum Focus {
    /// Show results from library code
    Lib,

    Test(
        /// Show results from a test
        #[bpaf(long("test"), argument("TEST"))]
        String,
    ),

    Bench(
        /// Show results from a benchmark
        #[bpaf(long("bench"), argument("BENCH"))]
        String,
    ),

    Example(
        /// Show results from an example
        #[bpaf(long("example"), argument("EXAMPLE"), complete(comp_examples))]
        String,
    ),

    Bin(
        /// Show results from a binary
        #[bpaf(long("bin"), argument("BIN"))]
        String,
    ),
}

fn main() {
    println!("{:#?}", options().run());
}
