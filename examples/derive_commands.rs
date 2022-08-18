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
