use crate::shared::{cargo_opts, package_and_testables, CargoOpts, PackageAndTestables};
use bpaf::*;
use std::process::Command;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command, short('c'), long("check"), generate(parse_check))]
/// Check a local package or parts of it and all of its dependencies for errors
pub struct Check {
    #[bpaf(external)]
    pub cargo_opts: CargoOpts,

    /// Package to check
    #[bpaf(external)]
    pub package_and_testables: PackageAndTestables,

    /// Check the library
    pub lib: bool,

    /// Check all the binaries
    pub bins: bool,

    /// Check all examples
    pub examples: bool,

    /// Check all tests
    pub tests: bool,

    /// Check all benchmarks
    pub benches: bool,
}

impl Check {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("check");
        self.package_and_testables.pass_to_cmd(cmd);
        self.cargo_opts.pass_to_cmd(cmd);
        pass_flag!(cmd, self.lib, "--lib");
        pass_flag!(cmd, self.bins, "--bins");
        pass_flag!(cmd, self.examples, "--examples");
        pass_flag!(cmd, self.benches, "--benches");
        pass_flag!(cmd, self.tests, "--tests");
    }
}
