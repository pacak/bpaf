use std::{ffi::OsString, sync::Mutex};

use bpaf::{positional, positional_os, Bpaf, CompleteDecor, Parser};
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand};
use once_cell::sync::Lazy;

// read cargo metadata
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Bin,
    Example,
}

#[derive(Debug, Clone)]
pub struct Exec<'a> {
    pub package: &'a str,
    pub name: &'a str,
    pub kind: Kind,
}

pub static EXECS: Lazy<Vec<Exec<'static>>> = Lazy::new(|| {
    let mut execs = Vec::new();
    for package in &METADATA.packages {
        for target in &package.targets {
            let kind;
            if target.kind[0] == "bin" {
                kind = Kind::Bin;
            } else if target.kind[0] == "example" {
                kind = Kind::Example;
            } else {
                continue;
            }
            execs.push(Exec {
                package: &package.name,
                name: &target.name,
                kind,
            });
        }
    }
    execs
});

pub static METADATA: Lazy<Metadata> = Lazy::new(|| {
    MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .no_deps()
        .exec()
        .unwrap()
});

#[derive(Debug, Default, Bpaf, Clone)]
#[bpaf(complete_style(CompleteDecor::VisibleGroup("== Cargo options")))]
pub struct CargoParams {
    #[bpaf(
        short,
        long,
        argument("SPEC"),
        complete(complete_package),
        map(remember_package),
        optional
    )]
    package: Option<String>,

    #[bpaf(external, map(remember_binary), optional)]
    binary: Option<Binary>,
    #[bpaf(map(remember_release))]
    release: bool,
    //    quiet: bool,
    //    verbose: usize,
}

// this exists so cauwugo can refer to that while trying to complete arguments
// for the binaries, this is not needed for usual argument parsing
pub static CARGO_PARAMS: Lazy<Mutex<CargoParams>> =
    Lazy::new(|| Mutex::new(CargoParams::default()));

#[derive(Debug, Clone, Bpaf)]
pub enum Binary {
    Bin(#[bpaf(long("bin"), argument("NAME"), complete(complete_binary))] String),
    Example(#[bpaf(long("example"), argument("NAME"), complete(complete_example))] String),
}

fn complete_binary(input: &String) -> Vec<(String, Option<String>)> {
    let mut res = Vec::new();
    for exe in EXECS.iter() {
        if exe.kind == Kind::Bin && exe.name.starts_with(input) {
            res.push((exe.name.to_string(), Some(format!("({})", exe.package))));
        }
    }
    res
}

fn complete_example(input: &String) -> Vec<(String, Option<String>)> {
    let mut res = Vec::new();
    let params = &CARGO_PARAMS.lock().unwrap();
    for exe in EXECS.iter() {
        if params.package.as_ref().map_or(false, |p| exe.package != p) {
            continue;
        }

        if exe.kind == Kind::Example && exe.name.starts_with(input) {
            res.push((exe.name.to_string(), Some(format!("({})", exe.package))));
        }
    }
    res
}

fn complete_package(input: &String) -> Vec<(String, Option<String>)> {
    let mut res = Vec::new();
    for pkg in &METADATA.packages {
        let name = &pkg.name;
        if name.starts_with(input) {
            res.push((name.clone(), None))
        }
    }
    res
}

pub fn cargo_command(
    cargo_command: &'static str,
    opts: &CargoParams,
    exec: Option<&Exec<'static>>,
) -> std::process::Command {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg(cargo_command);

    if let Some(exec) = &exec {
        cmd.arg("--package").arg(exec.package);
        match exec.kind {
            Kind::Bin => cmd.arg("--bin").arg(exec.name),
            Kind::Example => cmd.arg("--example").arg(exec.name),
        };
    } else {
        if let Some(package) = &opts.package {
            cmd.arg("--package").arg(package);
        }
        if let Some(binary) = &opts.binary {
            match binary {
                Binary::Bin(name) => cmd.arg("--bin").arg(name),
                Binary::Example(name) => cmd.arg("--example").arg(name),
            };
        }
    }
    cmd
}

fn complete_binary_args(input: &Vec<OsString>) -> Vec<(String, Option<String>)> {
    if input.is_empty() {
        return vec![(
            "--".to_owned(),
            Some("proceed to inner app arguments".to_owned()),
        )];
    }
    let mut cmd = cargo_command("run", &CARGO_PARAMS.lock().unwrap(), None);

    let output = cmd
        .arg("--quiet")
        .arg("--")
        .arg("--bpaf-complete-rev=2")
        .args(input.as_slice())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    let mut res = Vec::new();

    for line in std::str::from_utf8(&output.stdout).unwrap().lines() {
        res.push(match line.split_once('\t') {
            Some((arg, decor)) => (arg.to_owned(), Some(decor.to_owned())),
            None => (line.to_owned(), None),
        })
    }

    res
}

fn remember_package(package: String) -> String {
    CARGO_PARAMS.lock().unwrap().package = Some(package.clone());
    package
}

fn remember_binary(binary: Binary) -> Binary {
    CARGO_PARAMS.lock().unwrap().binary = Some(binary.clone());
    binary
}

fn remember_release(release: bool) -> bool {
    CARGO_PARAMS.lock().unwrap().release = release;
    release
}

fn child_process_args() -> impl Parser<Vec<OsString>> {
    positional_os("CHILD_ARG")
        .strict()
        .many()
        .complete(complete_binary_args)
}

#[allow(clippy::ptr_arg)]
fn complete_ws_binary(name: &String) -> Vec<(String, Option<String>)> {
    let mut res = Vec::new();
    let params = CARGO_PARAMS.lock().unwrap();
    for exec in EXECS.iter() {
        if !exec.name.starts_with(name) {
            continue;
        }
        if let Some(pkg) = &params.package {
            if exec.package != pkg {
                continue;
            }
        }
        if let Some(bin) = &params.binary {
            if match bin {
                Binary::Bin(name) => !(exec.kind == Kind::Bin && exec.name == name),
                Binary::Example(name) => !(exec.kind == Kind::Example && exec.name == name),
            } {
                continue;
            }
        }
        res.push((exec.name.to_owned(), None));
    }
    res
}

fn parse_ws_binary(name: String) -> Result<Exec<'static>, String> {
    let exec = EXECS
        .iter()
        .find(|e| e.name == name)
        .cloned()
        .ok_or_else(|| format!("{} is not a valid exec name", name))?;
    let params = &mut CARGO_PARAMS.lock().unwrap();
    params.package = Some(exec.package.to_owned());
    params.binary = Some(match exec.kind {
        Kind::Bin => Binary::Bin(exec.name.to_owned()),
        Kind::Example => Binary::Example(exec.name.to_owned()),
    });

    Ok(exec)
}

fn pick_binary() -> impl Parser<Exec<'static>> {
    positional("BIN")
        .help("binary or executable name available in a workspace")
        .complete(complete_ws_binary)
        .complete_style(CompleteDecor::VisibleGroup("== Workspace binaries"))
        .parse(parse_ws_binary)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command)]
    Run {
        #[bpaf(external)]
        cargo_params: CargoParams,

        #[bpaf(external(pick_binary), optional)]
        exec: Option<Exec<'static>>,

        #[bpaf(external(child_process_args))]
        args: Vec<OsString>,
    },
}

/*
    -q, --quiet                     Do not print cargo log messages
        --bin <NAME>                Name of the bin target to run
        --example <NAME>            Name of the example target to run
    -p, --package [<SPEC>...]       Package with the target to run
    -v, --verbose                   Use verbose output (-vv very verbose/build.rs output)
    -j, --jobs <N>                  Number of parallel jobs, defaults to # of CPUs
        --color <WHEN>              Coloring: auto, always, never
        --keep-going                Do not abort the build as soon as there is an error (unstable)
        --frozen                    Require Cargo.lock and cache are up to date
    -r, --release                   Build artifacts in release mode, with optimizations
        --locked                    Require Cargo.lock is up to date
        --profile <PROFILE-NAME>    Build artifacts with the specified profile
    -F, --features <FEATURES>       Space or comma separated list of features to activate
        --offline                   Run without accessing the network
        --all-features              Activate all available features
        --config <KEY=VALUE>        Override a configuration value (unstable)
        --no-default-features       Do not activate the `default` feature
    -Z <FLAG>                       Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for
                                    details
        --target <TRIPLE>           Build for the target triple
        --target-dir <DIRECTORY>    Directory for all generated artifacts
        --manifest-path <PATH>      Path to Cargo.toml
        --message-format <FMT>      Error format
        --unit-graph                Output build graph in JSON (unstable)
        --ignore-rust-version       Ignore `rust-version` specification in packages
        --timings[=<FMTS>...]       Timing output formats (unstable) (comma separated): html, json
    -h, --help                      Print help information
*/
