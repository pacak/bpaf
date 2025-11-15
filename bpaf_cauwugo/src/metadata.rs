use cargo_metadata::{CargoOpt, Metadata, MetadataCommand, Target};
use once_cell::sync::Lazy;
use std::process::Command;

pub static METADATA: Lazy<Metadata> = Lazy::new(|| {
    MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .no_deps()
        .exec()
        .unwrap()
});

pub fn bpaf_passthough_for(pkg: &str) -> bool {
    fn get(pkg: &str) -> Option<bool> {
        let meta = METADATA.workspace_metadata.get("cauwugo")?;
        let bpaf = meta.get("bpaf")?;

        if let Some(b) = bpaf.as_bool() {
            Some(b)
        } else if let Some(l) = bpaf.as_array() {
            for enabled_for in l.iter().filter_map(|l| l.as_str()) {
                if pkg == enabled_for {
                    return Some(true);
                }
            }
            Some(false)
        } else {
            Some(false)
        }
    }

    get(pkg).unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub enum MatchKind<'a> {
    Prefix(&'a str),
    Exact(&'a str),
    Any,
}

impl<'a> MatchKind<'a> {
    pub fn matches(&self, string: &str) -> bool {
        match self {
            MatchKind::Prefix(sel) => string.starts_with(sel),
            MatchKind::Exact(sel) => sel == &string,
            MatchKind::Any => true,
        }
    }

    pub fn exact(string: Option<&'static str>) -> Self {
        match string {
            Some(s) => Self::Exact(s),
            None => Self::Any,
        }
    }
}

/// all the matching targets using all the available information
pub fn matching_targets<'a>(
    package: MatchKind<'a>,
    name: MatchKind<'a>,
    kinds: &'static [&'static str],
) -> impl Iterator<Item = Exec> + 'a {
    METADATA
        .packages
        .iter()
        .filter(move |p| package.matches(&p.name))
        .flat_map(move |p| {
            p.targets
                .iter()
                .filter(move |t| {
                    name.matches(&t.name)
                        && t.kind
                            .first()
                            .is_some_and(|kind| kinds.contains(&kind.as_str()))
                })
                .filter_map(|t| match t.kind.first()?.as_str() {
                    "bin" => Some(Exec::Bin {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    "example" => Some(Exec::Example {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    "test" => Some(Exec::Test {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    "bench" => Some(Exec::Bench {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    "lib" => Some(Exec::Lib {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    "proc-macro" => Some(Exec::ProcMacro {
                        pkg: &p.name,
                        name: &t.name,
                    }),
                    _ => None,
                })
        })
}

#[derive(Debug, Clone, Copy)]
pub enum Exec {
    Bin {
        pkg: &'static str,
        name: &'static str,
    },
    Example {
        pkg: &'static str,
        name: &'static str,
    },
    Test {
        pkg: &'static str,
        name: &'static str,
    },
    Bench {
        pkg: &'static str,
        name: &'static str,
    },
    Lib {
        pkg: &'static str,
        name: &'static str,
    },
    ProcMacro {
        pkg: &'static str,
        name: &'static str,
    },
}

impl Exec {
    pub fn matches(&self, package: Option<&str>, target: &Target) -> bool {
        let (&pkg, &name, kind) = match self {
            Exec::Bin { pkg, name } => (pkg, name, "bin"),
            Exec::Example { pkg, name } => (pkg, name, "example"),
            Exec::Test { pkg, name } => (pkg, name, "test"),
            Exec::Bench { pkg, name } => (pkg, name, "bench"),
            Exec::Lib { pkg, name } => (pkg, name, "lib"),
            Exec::ProcMacro { pkg, name } => (pkg, name, "lib"),
        };
        name == target.name
            && package.is_none_or(|p| p == pkg)
            && target.kind.first().map(String::as_str) == Some(kind)
    }

    pub fn name(self) -> &'static str {
        match self {
            Exec::Bin { name, .. }
            | Exec::Example { name, .. }
            | Exec::Test { name, .. }
            | Exec::Bench { name, .. }
            | Exec::ProcMacro { name, .. }
            | Exec::Lib { name, .. } => name,
        }
    }

    pub fn pkg(self) -> &'static str {
        match self {
            Exec::Bin { pkg, .. }
            | Exec::Lib { pkg, .. }
            | Exec::Example { pkg, .. }
            | Exec::Test { pkg, .. }
            | Exec::ProcMacro { pkg, .. }
            | Exec::Bench { pkg, .. } => pkg,
        }
    }

    pub fn pass_to_cmd(&self, cmd: &mut Command) {
        match self {
            Exec::Bin { pkg, name } => cmd.args(["--package", pkg, "--bin", name]),
            Exec::Example { pkg, name } => cmd.args(["--package", pkg, "--example", name]),
            Exec::Test { pkg, name } => cmd.args(["--package", pkg, "--test", name]),
            Exec::Bench { pkg, name } => cmd.args(["--package", pkg, "--bench", name]),
            Exec::ProcMacro { pkg, .. } => cmd.args(["--package", pkg, "--lib"]),
            Exec::Lib { pkg, .. } => cmd.args(["--package", pkg, "--lib"]),
        };
    }
}
