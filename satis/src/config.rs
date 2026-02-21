use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::Command;

#[derive(Debug)]
pub struct Binary {
    /// Binary name - `satis` detects it in shell prompt
    pub name: String,
    /// A command required to compile the binary
    /// If not set - binary assumed to be there
    ///

    /// I need to create it several times
    pub compile: Option<Command>,
    /// Absolute path to the directory with the binary
    pub path: OsString,
    /// An extra flag/argument for each supported shell to dump shell completion to `stdout`
    ///
    /// for example, `cargo-asm --bpaf-complete-style-bash` will produce shell completions for bash,
    /// String will contain `--bpaf-complete-style-bash` part
    pub complete: BTreeMap<Shell, MkComplete>,
}

// TODO - this needs to be ensmartened...
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MkComplete {
    /// app gets executed with this extra flag to get the completion test
    pub arg: String,
    /// then this bit gets added to the config (or not, if it's zsh)
    pub to_config: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Copy, Clone)]
enum Cli {
    Bpaf,
}

impl Cli {
    fn populate(self) -> BTreeMap<Shell, MkComplete> {
        match self {
            Cli::Bpaf => [
                (
                    Shell::Bash,
                    MkComplete {
                        arg: String::from("--bpaf-complete-style-bash"),
                        to_config: String::new(),
                    },
                ),
                (
                    Shell::Zsh,
                    MkComplete {
                        arg: String::from("--bpaf-complete-style-zsh"),
                        to_config: String::new(),
                    },
                ),
            ]
            .into(),
        }
    }
}

pub fn config_from_cargo() -> anyhow::Result<Option<BTreeMap<String, Binary>>> {
    use cargo_metadata::TargetKind as TK;
    if !std::fs::exists("Cargo.toml")? {
        return Ok(None);
    }

    let meta = cargo_metadata::MetadataCommand::new().no_deps().exec()?;

    let mut cli_is = None;
    for p in &meta.packages {
        // won't work for crates with a zoo of parsers
        if p.name == "bpaf" || p.dependencies.iter().any(|dep| dep.name == "bpaf") {
            cli_is = Some(Cli::Bpaf);
        }
        if cli_is.is_some() {
            break;
        }
    }

    let mut res = BTreeMap::new();
    for p in &meta.packages {
        for t in &p.targets {
            let [kind] = &t.kind[..] else {
                continue;
            };

            let mut path = std::env::current_dir()?;
            path.push(match kind {
                TK::Bin => "target/release",
                TK::Example => "target/release/examples",
                _ => continue,
            });
            let mut compile = Command::new("cargo");
            compile
                .arg("build")
                .arg("--quiet")
                .arg("--release")
                .args(["-p", &p.name]);
            match kind {
                TK::Bin => compile.args(["--bin", &t.name]),
                TK::Example => compile.args(["--example", &t.name]),
                _ => unreachable!(),
            };
            let bin = Binary {
                name: t.name.clone(),
                compile: Some(compile),
                path: OsString::from(path),
                complete: cli_is.map_or_else(BTreeMap::new, Cli::populate),
            };
            res.insert(bin.name.clone(), bin);
        }
    }
    Ok(Some(res))
}
