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

fn feature_if() -> Parser<Option<String>> {
    positional_if("FEATURE", |v| !is_version(v))
}

fn version_if() -> Parser<Option<String>> {
    positional_if("VERSION", is_version)
}

fn is_version(v: &str) -> bool {
    v.chars().all(|c| c.is_numeric())
}

fn main() {
    println!("{:?}", action().run());
}
