use crate::shared::{cargo_opts, parse_package, CargoOpts};
use bpaf::*;
use std::process::Command;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command, generate(parse_clean))]
/// Remove artifacts that cargo has generated in the past
pub struct Clean {
    #[bpaf(external)]
    pub cargo_opts: CargoOpts,

    /// Package to clean
    #[bpaf(external)]
    pub package: Option<&'static str>,
}

impl Clean {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("clean");
        pass_arg!(cmd, self.package, "--package");
        self.cargo_opts.pass_to_cmd(cmd);
    }
}

fn package() -> impl Parser<Option<&'static str>> {
    parse_package("Package to clean artifacts for")
}
