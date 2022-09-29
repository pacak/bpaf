use crate::{
    metadata::{bpaf_passthough_for, Exec},
    opts::parse_runnable,
    remember_opt, remember_req,
    shared::{cargo_opts, parse_package, CargoOpts},
};
use bpaf::*;
use std::{cell::RefCell, ffi::OsString, process::Command, rc::Rc};

#[derive(Debug, Clone)]
pub struct Run {
    pub cargo_opts: CargoOpts,
    pub package: Option<&'static str>,
    pub runnable: Exec,
    pub args: Vec<OsString>,
}

impl Run {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("run");
        self.cargo_opts.pass_to_cmd(cmd);
        if let Some(p) = self.package {
            cmd.args(["--package", p]);
        }
        self.runnable.pass_to_cmd(cmd);
        cmd.arg("--");
        cmd.args(&self.args);
    }
}

/// This one is a hack. While it is easy to get the lines from child process - it's harder
/// to convert them to the expected output. Luckily all we want is to dump them in the same format
/// as we got them and nothing else. So not capturing the output + exit is the obvious choice
fn complete_binary_args(input: &[OsString], exec: Option<Exec>) -> Vec<(String, Option<String>)> {
    if input.is_empty() {
        return vec![(
            "--".to_owned(),
            Some("proceed to inner app arguments".to_owned()),
        )];
    }

    if !bpaf_passthough_for(exec.map_or("", Exec::pkg)) {
        return vec![(
            "<ARG>".to_owned(),
            Some("argument completion requires bpaf passthough enabled".to_owned()),
        )];
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    if let Some(exec) = exec {
        exec.pass_to_cmd(&mut cmd);
    }

    let revision = std::env::args_os().nth(1).unwrap();

    cmd.arg("--quiet").arg("--").arg(revision).args(input);

    let output = cmd
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output();

    match output {
        Ok(ok) => {
            if ok.status.success() {
                std::process::exit(0);
            } else {
                eprint!("{}", std::str::from_utf8(&ok.stderr).unwrap());
                std::process::exit(ok.status.code().unwrap_or(1));
            }
        }
        Err(err) => {
            panic!("{}", err);
        }
    }
}

pub fn parse_run() -> impl Parser<Run> {
    let cur_pkg = Rc::new(RefCell::new(None));
    let cur_exec = Rc::new(RefCell::new(None));

    let package = remember_opt(parse_package("Package with the target to run"), &cur_pkg);
    let runnable = remember_req(parse_runnable(cur_pkg), &cur_exec);

    let args = positional::<OsString>("args")
        .help("Cauwugo will pass arguments after -- to the child process")
        .strict()
        .many()
        .complete(move |input| complete_binary_args(input, *cur_exec.borrow()));

    construct!(Run {
        cargo_opts(),
        package,
        runnable,
        args,
    })
    .to_options()
    .descr("Run a binary or example of the local package")
    .command("run")
    .short('r')
}
