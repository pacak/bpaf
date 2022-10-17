use bpaf::*;
use std::{cell::RefCell, process::Command, rc::Rc};

use crate::{
    metadata::{Exec, METADATA},
    opts::{parse_testable, suggest_available},
};

/// A set of options somewhat shared between different cargo commands
///
/// Some are redundant :)
#[derive(Debug, Clone, Bpaf)]
pub struct CargoOpts {
    /// Use release mode with optimizations
    #[bpaf(short, long)]
    pub release: bool,

    /// Don't print cargo log messages
    #[bpaf(short, long)]
    pub quiet: bool,

    /// Require Cargo.lock and cache are up to date
    pub frozen: bool,

    /// Require Cargo.lock is up to date
    pub locked: bool,

    /// Run without accessing the network
    pub offline: bool,

    /// Number of parallel jobs, defaults to # of CPUs
    pub jobs: Option<usize>,

    /// Do not activate the `default` feature
    pub no_default_features: bool,

    /// Activate all available features
    pub all_features: bool,

    #[bpaf(argument("TRIPLE"), complete(complete_available_target), optional)]
    /// Build for the target triple
    pub target: Option<String>,
}

impl CargoOpts {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        pass_flag!(cmd, self.release, "--release");
        pass_flag!(cmd, self.quiet, "--quiet");
        pass_flag!(cmd, self.frozen, "--frozen");
        pass_flag!(cmd, self.locked, "--locked");
        pass_flag!(cmd, self.offline, "--offline");
        pass_flag!(cmd, self.no_default_features, "--no_default_features");
        pass_flag!(cmd, self.all_features, "--all-features");
        pass_arg!(cmd, self.jobs.map(|j| j.to_string()), "--jobs");
    }
}

#[derive(Debug, Clone)]
pub struct PackageAndTestables {
    pub package: Option<&'static str>,
    pub testables: Vec<Exec>,

    pub features: Vec<String>,
}

impl PackageAndTestables {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        for t in &self.testables {
            t.pass_to_cmd(cmd)
        }
        pass_arg!(cmd, self.package, "--package");
    }
}

pub fn parse_package(help: &'static str) -> impl Parser<Option<&'static str>> {
    short('p')
        .long("package")
        .help(help)
        .argument::<String>("NAME")
        .complete(|i| suggest_available(&[i], METADATA.packages.iter().map(|p| p.name.as_str())))
        .parse(|i| {
            METADATA
                .packages
                .iter()
                .find_map(|p| {
                    if p.name == i {
                        Some(p.name.as_str())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("{} is not a known package", i))
        })
        .optional()
}

pub fn parse_features(cur_pkg: Rc<RefCell<Option<&'static str>>>) -> impl Parser<Vec<String>> {
    short('F')
        .long("feature")
        .help("Feature to activate, one at a time")
        .argument("FEATURES")
        .many()
        .complete(move |i| {
            match cur_pkg
                .borrow()
                .and_then(|p| METADATA.packages.iter().find(|pkg| pkg.name == p))
            {
                Some(pkg) => suggest_available(i, pkg.features.keys().map(|s| s.as_str())),
                None => vec![("<FEATURE>".to_owned(), None)],
            }
        })
}

pub fn package_and_testables() -> impl Parser<PackageAndTestables> {
    let cur_pkg = Rc::new(RefCell::new(None));
    let testables = parse_testable(cur_pkg.clone()).many();
    let features = parse_features(cur_pkg.clone());
    let package = parse_package("Package to check").map(move |p| {
        *cur_pkg.borrow_mut() = p;
        p
    });

    construct!(PackageAndTestables {
        package,
        testables,
        features
    })
}

#[allow(clippy::ptr_arg)]
fn complete_available_target(input: &String) -> Vec<(String, Option<String>)> {
    let mut cmd = Command::new("rustup");
    cmd.args(["target", "list", "--installed"]);
    let output = cmd.output().expect("Couldn't run rustup");
    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|l| {
                if l.starts_with(input) {
                    Some((l.to_owned(), None))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        eprint!("{}", std::str::from_utf8(&output.stderr).unwrap());
        panic!("Couldn't get list of installed targets from rustup");
    }
}
