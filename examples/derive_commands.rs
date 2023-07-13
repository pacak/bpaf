//! Snippet from cargo-hackerman crate, shows how to use derive to parse commands and
//! conditional skip for options
//!
//! Command explain takes 3 parameters: required crate name and optional feature and crate version,
//! user is allowed to omit either field. This example uses simplified is_version, in practice you would
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
    // here feature starts as any string on a command line that does not start with a dash
    positional::<String>("FEATURE")
        // guard restricts it such that it can't be a valid version
        .guard(move |s| !is_version(s), "")
        // last two steps describe what to do with strings in this position but are actually
        // versions.
        // optional allows parser to represent an ignored value with None
        .optional()
        // and catch lets optional to handle parse failures coming from guard
        .catch()
}

fn version_if() -> impl Parser<Option<String>> {
    positional::<String>("VERSION")
        .guard(move |s| is_version(s), "")
        .optional()
        .catch()
}

fn is_version(v: &str) -> bool {
    v.chars().all(|c| c.is_numeric())
}

fn main() {
    println!("{:?}", action().fallback_to_usage().run());
}
