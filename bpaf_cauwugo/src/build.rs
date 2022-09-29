use crate::shared::{cargo_opts, package_and_testables, CargoOpts, PackageAndTestables};
use bpaf::*;
use std::process::Command;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command, short('b'), long("build"), generate(parse_build))]
/// Compile a local package or parts of it and all of its dependencies
pub struct Build {
    #[bpaf(external)]
    pub cargo_opts: CargoOpts,

    /// Package to check
    #[bpaf(external)]
    pub package_and_testables: PackageAndTestables,

    /// Build the library
    pub lib: bool,

    /// Build all the binaries
    pub bins: bool,

    /// Build all examples
    pub examples: bool,

    /// Build all tests
    pub tests: bool,

    /// Build all benchmarks
    pub benches: bool,
}

impl Build {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("build");
        self.package_and_testables.pass_to_cmd(cmd);
        self.cargo_opts.pass_to_cmd(cmd);
        pass_flag!(cmd, self.lib, "--lib");
        pass_flag!(cmd, self.bins, "--bins");
        pass_flag!(cmd, self.examples, "--examples");
        pass_flag!(cmd, self.benches, "--benches");
        pass_flag!(cmd, self.tests, "--tests");
    }
}
