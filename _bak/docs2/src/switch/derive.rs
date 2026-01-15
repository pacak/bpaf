//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Produce verbose output
    // bpaf uses `switch` for `bool` fields in named
    // structs unless consumer attribute is present.
    // But it is also possible to give it explicit
    // consumer annotation to serve as a reminder:
    // #[bpaf(short, long, switch)]
    #[bpaf(short, long)]
    verbose: bool,

    #[bpaf(flag(true, false))]
    /// Build artifacts in release mode
    release: bool,

    /// Do not activate default features
    // default_features uses opposite values,
    // producing `true` when value is absent
    #[bpaf(long("no-default-features"), flag(false, true))]
    default_features: bool,
}
