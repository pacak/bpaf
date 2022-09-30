use crate::shared::parse_package;
use bpaf::*;
use std::{path::PathBuf, process::Command};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command, generate(parse_add))]
/// Add dependencies to a Cargo.toml manifest file
pub struct Add {
    #[bpaf(external)]
    pub package: Option<&'static str>,

    /// Add as dev dependency
    pub dev: bool,

    /// Add as build dependency
    pub build: bool,

    /// Don't actually write the manifest
    pub dry_run: bool,

    #[bpaf(external)]
    pub source: Source,
}

#[derive(Debug, Clone, Bpaf)]
pub enum Source {
    Git {
        #[bpaf(long, argument("URI"))]
        /// Git repository location
        git: String,

        #[bpaf(long, argument("BRANCH"))]
        /// Git branch to download the crate from
        branch: Option<String>,

        #[bpaf(long, argument("TAG"))]
        /// Git tag to download the crate from
        tag: Option<String>,

        #[bpaf(long, argument("REV"))]
        /// Git reference to download the crate from
        rev: Option<String>,

        /// Package name
        #[bpaf(positional("DEP"))]
        // we can go to the URL and check what the names are available there...
        name: Option<String>,
    },
    Local {
        /// Filesystem path to local crate to add
        path: PathBuf,

        /// Package name
        #[bpaf(positional("DEP"))]
        // and ask cargo for names available there...
        name: Option<String>,
    },
    Crates {
        /// Package registry to use instead of crates.io
        registry: Option<String>,

        /// Package name
        #[bpaf(positional("DEP"), complete(complete_available_package))]
        name: String,
    },
}

/// ask cargo search for all the available packages matching the name
///
/// TODO: do some filtering - squatted names and such don't belong here
fn complete_available_package(name: &String) -> Vec<(String, Option<String>)> {
    if name.is_empty() {
        return vec![("<PACKAGE>".to_string(), None)];
    }
    let mut cmd = Command::new("cargo");
    cmd.args(["search", "--limit", "100"]); // greedy cargo will print only 100 or so at most :(
    cmd.arg(name);
    let output = cmd.output().expect("Couldn't run cargo search?");

    if output.status.code() == Some(0) {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                if !line.starts_with(name) {
                    return None;
                }
                let (full_package, descr) = line.split_once(" # ")?;
                let package = full_package.split(' ').next()?;
                Some((package.to_owned(), Some(descr.to_owned())))
            })
            .collect::<Vec<_>>()
    } else {
        panic!("{:?}", output);
    }
}

impl Add {
    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        cmd.arg("add");
        pass_arg!(cmd, self.package, "--package");
        pass_flag!(cmd, self.dev, "--dev");
        pass_flag!(cmd, self.dry_run, "--dry-run");
        pass_flag!(cmd, self.build, "--build");
        match &self.source {
            Source::Git {
                git,
                branch,
                tag,
                rev,
                name,
            } => {
                pass_req_arg!(cmd, git, "--git");
                pass_arg!(cmd, branch, "--branch");
                pass_arg!(cmd, tag, "--tag");
                pass_arg!(cmd, rev, "--rev");
                pass_pos!(cmd, name);
            }
            Source::Local { path, name } => {
                pass_req_arg!(cmd, path, "--path");
                pass_pos!(cmd, name);
            }
            Source::Crates { registry, name } => {
                cmd.arg(name);
                pass_arg!(cmd, registry, "--registry");
            }
        }
    }
}

fn package() -> impl Parser<Option<&'static str>> {
    parse_package("Package to add a new dependency to")
}
