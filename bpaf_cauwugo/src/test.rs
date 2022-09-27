use std::{cell::RefCell, process::Command, rc::Rc};

use bpaf::*;

use crate::{
    metadata::{matching_targets, Exec, MatchKind},
    opts::complete_target_kind,
    shared::{cargo_opts, package_and_testables, CargoOpts, PackageAndTestables},
};

#[derive(Debug, Clone)]
pub enum Test {
    All(Tests),
    Specific(Specific),
}

impl Test {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("test");
        match self {
            Test::All(tests) => tests.pass_to_cmd(cmd),
            Test::Specific(spec) => {
                spec.test.pass_to_cmd(cmd);
                if let Some(name) = &spec.name {
                    cmd.args(["--", "--exact", name]);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(parse_test_general))]
pub struct Tests {
    #[bpaf(external)]
    pub cargo_opts: CargoOpts,

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

    #[bpaf(external)]
    pub package_and_testables: PackageAndTestables,
}

impl Tests {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        self.package_and_testables.pass_to_cmd(cmd);
        self.cargo_opts.pass_to_cmd(cmd);
        pass_flag!(cmd, self.lib, "--lib");
        pass_flag!(cmd, self.bins, "--bins");
        pass_flag!(cmd, self.examples, "--examples");
        pass_flag!(cmd, self.benches, "--benches");
        pass_flag!(cmd, self.tests, "--tests");
    }
}

#[derive(Debug, Clone)]
pub struct Specific {
    pub test: Exec,
    pub name: Option<String>,
}

const TESTABLE: &[&str] = &["test", "lib", "proc-macro"];

fn complete_subtest_name(input: &str, current_test: Option<Exec>) -> Vec<(String, Option<String>)> {
    let current_test = match current_test {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["test", "--quiet"]);
    current_test.pass_to_cmd(&mut cmd);
    cmd.args(["--", "--list"]);

    let output = cmd.output().unwrap();
    if let Some(0) = output.status.code() {
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .filter_map(|l| {
                let test_name = l.strip_suffix(": test")?;
                if test_name.starts_with(input) {
                    Some((test_name.to_owned(), None))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
        std::process::exit(1)
    }
}

pub fn parse_specific() -> impl Parser<Test> {
    let cur_exec = Rc::new(RefCell::new(None));
    let cur_exec2 = cur_exec.clone();
    let test = positional::<String>("TEST")
        .help("Test file name")
        .complete(move |i| complete_target_kind(&[i], None, TESTABLE))
        .parse::<_, _, String>(move |name| {
            let mut iter = matching_targets(MatchKind::Any, MatchKind::Exact(&name), TESTABLE);
            match (iter.next(), iter.next()) {
                (None, _) => Err(format!("{} is not a known test name", name)),
                (Some(_), Some(_)) => Err(format!("{} is not a unique test name", name)),
                (Some(exec), None) => {
                    *cur_exec.borrow_mut() = Some(exec);
                    Ok(exec)
                }
            }
        })
        .complete_style(CompleteDecor::VisibleGroup("Bins, tests, examples"));

    let name = positional::<String>("NAME")
        .help("Test name in a file")
        .complete(move |i| complete_subtest_name(i, *cur_exec2.borrow()))
        .complete_style(CompleteDecor::VisibleGroup("Available test names"))
        .optional();

    construct!(Specific { test, name }).map(Test::Specific)
}

pub fn parse_test() -> impl Parser<Test> {
    let general = parse_test_general().map(Test::All);
    construct!([parse_specific(), general])
        .to_options()
        .descr("Execute unit and integration tests")
        .command("test")
        .short('t')
}
