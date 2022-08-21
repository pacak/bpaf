//! Snippet from cargo-hackerman crate, shows how to use derive to parse commands and
//! "positional_if" pattern.
//!
//! Command explain takes 3 parameters: required crate name and optional feature and crate version,
//! user is allowed to omitt either field. This example uses simplified is_version, in practice youwould
//! would use semver crate.
//!
//! End user would be able to run commands like
//!
//! ```console
//! $ cargo hackerman explain random 314
//! > krate: "random", feature: None, version: Some(314"),
//! $ cargo hackerman explain serde derive
//! > krate: "serde", feature: Some("derive"), version: None
//! ```

use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options("hackerman"))]
pub enum Action {
    #[bpaf(command("explain"))]
    Explain {
        #[bpaf(positional("CRATE"))]
        krate: String,
        #[bpaf(external(feature_if))]
        feature: Option<String>,
        #[bpaf(external(version_if))]
        version: Option<String>,
    },
}

fn feature_if() -> impl Parser<Option<String>> {
    positional("FEATURE")
        .guard(move |s| !is_version(s), "")
        .optional()
}

fn version_if() -> impl Parser<Option<String>> {
    positional("VERSION")
        .guard(move |s| is_version(s), "")
        .optional()
}

fn is_version(v: &str) -> bool {
    v.chars().all(|c| c.is_numeric())
}

fn main() {
    println!("{:?}", action().run());
}
