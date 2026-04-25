use anyhow::Context;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};
use tempdir::TempDir;
pub use term::Terminal;

mod config;
mod term;
pub use config::config_from_cargo;

mod cache {
    use crate::config::Shell;
    use anyhow::Context as _;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::path::PathBuf;

    fn cache_dir() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_default();
        // TODO - this relies on it being a cargo app
        path.push("target/satis_cache");
        path
    }

    fn cache_key(app: &str, prompt: &str, shell: &Shell) -> u64 {
        let mut hasher = DefaultHasher::new();
        app.hash(&mut hasher);
        prompt.hash(&mut hasher);
        shell.hash(&mut hasher);
        hasher.finish()
    }

    fn cache_path(app: &str, prompt: &str, shell: &Shell) -> PathBuf {
        let mut path = cache_dir();
        path.push(format!("{:x}.bin", cache_key(app, prompt, shell)));
        path
    }

    pub(crate) fn load_cached(
        app: &str,
        prompt: &str,
        shell: &Shell,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let path = cache_path(app, prompt, shell);
        match std::fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("reading cache file {path:?}")),
        }
    }

    pub(crate) fn save_cached(
        app: &str,
        prompt: &str,
        shell: &Shell,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let path = cache_path(app, prompt, shell);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating cache dir {parent:?}"))?;
        }
        std::fs::write(&path, data).with_context(|| format!("writing cache file {path:?}"))
    }

    pub fn evict_old_cache_entries() {
        let dir = cache_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return;
        };

        let cutoff =
            std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(86400));

        for entry in entries {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let Ok(modified) = metadata.modified() else {
                continue;
            };

            if let Some(cutoff) = cutoff
                && modified < cutoff
            {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

pub use crate::cache::*;

use crate::config::{Binary, Shell};

pub fn prepare_binaries(
    mds: &[Md],
    binaries: &mut BTreeMap<String, Binary>,
    verbose: bool,
) -> anyhow::Result<()> {
    // we need a `&mut Binary` to build it (limitations of `Command` API)
    // One way around this is to collect the names first and then build the binaries
    let names = mds
        .iter()
        .flat_map(Md::snippets)
        .map(|snip| {
            let bin = snip.bin();
            if binaries.contains_key(bin) {
                Ok(bin)
            } else {
                anyhow::bail!("Snippet requires unknown binary {bin:?}")
            }
        })
        .collect::<anyhow::Result<BTreeSet<_>>>()?;

    for name in names {
        let bin = binaries.get_mut(name).unwrap();
        let Some(cmd) = bin.compile.as_mut() else {
            continue;
        };
        if verbose {
            println!("Compiling {name}");
        }

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                println!("\x1b[0;31m{stderr}\x1b[0;0m");
            }
            anyhow::bail!(
                "Failed to build {cmd:?} (exit code: {:?})",
                output.status.code()
            );
        }
    }
    Ok(())
}

// 1. make home dir that would normalize the behavior
// 2. set PATH to include the binary under test
// 3. setup the completion
pub(crate) struct ShellInstance {
    pub(crate) tempdir: TempDir,
    pub(crate) run_shell: Command,
}

// Binary - information about the binary - path and instructions to compile.
// Shell - enum Zsh/Bash/Fish, given Binary can generate MkComplete.
// ShellInstance - prepared tempdir + a command to spawn the test.
// MkComplete - instructions to update the tempdir in ShellInstance with shell completion.
//              gets deserialized from a config or gets generated from Cargo.toml
// Terminal - pty with a shell running inside, takes input, produces outputs
// Cli - currently just bpaf. helps to fill in missing details in Binary
//
// Shell + Binary = MkComplete

pub const BASH_COMP: &str = "/usr/share/bash-completion/bash_completion";
impl Shell {
    pub(crate) fn prepare(self, binary: &Binary) -> anyhow::Result<ShellInstance> {
        let prefix = format!("shell-tester-{self:?}").to_lowercase();
        let tempdir = TempDir::new(&prefix)?;
        let home = tempdir.as_ref();

        let Some(mk_comp) = binary.complete.get(&self) else {
            anyhow::bail!("test requires {self:?}");
        };

        let mut fetch_script_cmd = Command::new(&binary.name);

        let mut path = binary.path.clone();
        if let Some(cur) = std::env::var_os("PATH") {
            path.push(":");
            path.push(cur);
        }
        fetch_script_cmd.arg(&mk_comp.arg);
        fetch_script_cmd.env("PATH", &path);

        let output = fetch_script_cmd
            .output()
            .context("fetching the completion script")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                println!("\x1b[0;31m{stderr}\x1b[0;0m");
            }
            anyhow::bail!(
                "Failed to fetch completion script (exit code: {:?})",
                output.status.code()
            );
        }
        let script = output.stdout;

        let mut run_shell = Command::new(self);
        run_shell.env_clear();
        run_shell.env("TERM", "xterm");
        run_shell.env("LC_ALL", "C");
        run_shell.env("PATH", &path);
        run_shell.env("HOME", home);

        match self {
            Shell::Bash => {
                use std::io::Write;
                if !std::fs::exists(BASH_COMP)? {
                    anyhow::bail!("Couldn't find {BASH_COMP} required for bash completions!");
                }

                let bashrc = home.join(".bashrc");
                let mut rc = std::fs::File::create(bashrc)?;
                writeln!(&rc, ". {BASH_COMP}")?;

                rc.write_all(&script)?;
                writeln!(&rc, "{}", mk_comp.to_config)?;

                let inputrc = home.join(".inputrc");
                std::fs::write(&inputrc, "")?;

                run_shell.env("PS1", "bash$ ");
            }
            Shell::Zsh => {
                use std::io::Write;
                let zshrc = home.join(".zshrc");
                let mut rc = std::fs::File::create(zshrc)?;

                rc.write_all(b"fpath=($fpath $HOME/.zsh)\n")?;
                rc.write_all(b"autoload -U +X compinit && compinit -i\n")?;
                rc.write_all(b"bindkey '^I' expand-or-complete-prefix\n")?;

                let app = &binary.name;

                std::fs::create_dir(home.join(".zsh"))?;
                let mut comp = std::fs::File::create(
                    home.join(".zsh").join(PathBuf::from(format!("_{app}"))),
                )?;
                comp.write_all(&script)?;

                run_shell.env("PS1", "zsh%% ");
            }
            Shell::Fish => todo!(),
        }
        Ok(ShellInstance { tempdir, run_shell })
    }
}

impl AsRef<OsStr> for Shell {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Snippet {
    /// Shell to use in this snippet
    shell: Shell,
    /// What user typed
    ///
    /// Prompts with `<TAB>` present are completion tests, without it are output tests
    prompt: String,
    /// Expected to output
    expected: String,
    stage: Stage,
}
#[derive(Debug, Clone)]
pub enum Stage {
    Pending,
    Matches,
    Mismatch { actual: String },
}

impl Snippet {
    fn new() -> Self {
        Self {
            shell: Shell::Bash,
            prompt: String::new(),
            expected: String::new(),
            stage: Stage::Pending,
        }
    }

    /// Get the executable name from the command line
    ///
    /// Executables with spaces, quoted, escaped or anything similar are not real
    pub fn bin(&self) -> &str {
        self.prompt
            .split_whitespace()
            .next()
            .unwrap_or(&self.prompt)
    }
}

#[derive(Debug, Clone)]
enum Chunk {
    Text(String),
    Chunk(Snippet),
}

impl Default for Chunk {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Shell::Bash => "bash$",
            Shell::Zsh => "zsh%",
            Shell::Fish => "fish>",
        })
    }
}

impl std::fmt::Display for Snippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Snippet {
            shell,
            prompt,
            expected,
            stage,
        } = self;

        writeln!(f, "```console")?;
        writeln!(f, "{shell} {prompt}")?;

        const RED: &str = "\x1b[0;31m";
        const GREEN: &str = "\x1b[0;32m";
        const RESET: &str = "\x1b[0;0m";
        match stage {
            Stage::Pending => writeln!(f, "... check pending")?,
            Stage::Matches => writeln!(f, "{GREEN}{expected}{RESET}")?,
            Stage::Mismatch { actual } => {
                for mm in diff::lines(expected, actual) {
                    match mm {
                        diff::Result::Left(s) => writeln!(f, "{RED}-{s}{RESET}")?,
                        diff::Result::Both(s, _) => writeln!(f, " {s}")?,
                        diff::Result::Right(s) => writeln!(f, "{GREEN}+{s}{RESET}")?,
                    }
                }
            }
        }
        writeln!(f, "```")
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chunk::Text(t) => write!(f, "{t}"),
            Chunk::Chunk(snippet) => write!(f, "{snippet}"),
        }
    }
}

#[derive(Debug)]
pub struct Md {
    path: PathBuf,
    chunks: Vec<Chunk>,
    active: Option<BTreeSet<usize>>,
}

impl std::fmt::Display for Md {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.path.to_string_lossy())?;
        for chunk in &self.chunks {
            write!(f, "{chunk}")?
        }
        Ok(())
    }
}

impl Shell {
    fn parse(line: &str) -> anyhow::Result<(Shell, &str)> {
        let Some((shell, rest)) = line.split_once(|c: char| ['$', '>', '%'].contains(&c)) else {
            anyhow::bail!(
                "Line {line:?} should start with a shell followed by a `$ `, `> ` or `%`"
            );
        };
        let shell = match shell {
            "bash" => Shell::Bash,
            "fish" => Shell::Fish,
            "zsh" => Shell::Zsh,
            _ => anyhow::bail!("Only bash, fish and zsh are supported"),
        };
        Ok((shell, rest.trim_start()))
    }
}

impl Md {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let payload = std::fs::read_to_string(path)
            .with_context(|| format!("Reading problem definition from {path:?}"))?;

        let mut chunks = Vec::new();

        #[derive(Clone, Copy)]
        enum SnipStage {
            /// Got `\`\`\`console` bit, expecting the prompt
            Shell,
            /// Got `bash$ hello`, what follows next is the expected output, if present
            Expected,
        }

        let mut cur = Default::default();
        let mut snip_stage = SnipStage::Shell;
        for (ix, line) in payload.lines().enumerate() {
            match &mut cur {
                Chunk::Text(text) => {
                    if line == "```console" {
                        chunks.push(std::mem::take(&mut cur));
                        cur = Chunk::Chunk(Snippet::new());
                        snip_stage = SnipStage::Shell;
                    } else {
                        text.push_str(line);
                        text.push('\n');
                    }
                }

                Chunk::Chunk(Snippet {
                    expected,
                    shell,
                    prompt,
                    stage: _,
                }) => match snip_stage {
                    SnipStage::Shell => {
                        if line == "```" {
                            anyhow::bail!(
                                "Expected shell at {path}:{ix}, got ```",
                                path = path.to_string_lossy()
                            );
                        } else {
                            let (sh, pr) = Shell::parse(line)?;
                            *shell = sh;
                            *prompt = pr.to_string();
                            snip_stage = SnipStage::Expected;
                        }
                    }
                    SnipStage::Expected => {
                        if line == "```" {
                            expected.truncate(expected.trim_end().len());
                            chunks.push(std::mem::take(&mut cur));
                        } else {
                            expected.push_str(line);
                            expected.push('\n');
                        }
                    }
                },
            }
        }

        if let Chunk::Chunk(_) = &cur {
            chunks.push(std::mem::take(&mut cur));
        }

        let active = file.indices.as_ref().map(|xs| xs.iter().copied().collect());

        Ok(Md {
            path: PathBuf::from(path),
            chunks,
            active,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Shell;
    #[test]
    fn shell_bash() {
        let (sh, ac) = Shell::parse("bash$ ls -<TAB>").unwrap();
        assert_eq!(sh, Shell::Bash);
        assert_eq!(ac, "ls -<TAB>");
    }

    #[test]
    fn shell_fish() {
        let (sh, ac) = Shell::parse("fish> ls -<TAB>").unwrap();
        assert_eq!(sh, Shell::Fish);
        assert_eq!(ac, "ls -<TAB>");
    }

    #[test]
    fn shell_zsh() {
        let (sh, ac) = Shell::parse("zsh% ls -<TAB>").unwrap();
        assert_eq!(sh, Shell::Zsh);
        assert_eq!(ac, "ls -<TAB>");
    }
}

pub fn op() -> impl bpaf::Parser<Op> {
    use bpaf::*;
    let timeout = std::time::Duration::from_millis(5000);
    let force = short('f')
        .long("force")
        .help("Check with 5000ms timeout")
        .req_flag(Op::Timeout { timeout });

    let custom = short('t')
        .long("timeout")
        .help("Check with a custom timeout (in ms)")
        .argument::<u64>("MS")
        .map(|timeout| Op::Timeout {
            timeout: std::time::Duration::from_millis(timeout),
        });
    let timeout = std::time::Duration::from_millis(300);
    construct!([force, custom]).fallback(Op::Timeout { timeout })
}

#[derive(Debug, Clone, Copy)]
/// Operation to perform
///
/// `Satis` will create the Markdown expansion if it's absent or will perform
/// of the checks. They both fail on the first mismatch, but condition
pub enum Op {
    /// Success `timeout` after the last update as long as output matches expectations
    ///
    /// This check might catch some unexpected output that takes some time to show up
    Timeout {
        /// Timeout in ms
        timeout: std::time::Duration,
    },
}

impl Shell {
    fn started(self) -> &'static str {
        match self {
            Shell::Bash => "bash$ ",
            Shell::Zsh => "zsh% ",
            Shell::Fish => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinarySource {
    CargoExample(String),
    CargoBin(String),
    Path(String, PathBuf),
}

impl Snippet {
    pub fn check(&mut self, binary: &Binary, op: Op) -> anyhow::Result<bool> {
        let prompt = self.prompt.replace("<TAB>", "\t");
        if prompt == self.prompt {
            println!("Ignoring the execution test for now");
            return Ok(false);
        }

        let cache = load_cached(&binary.name, &self.prompt, &self.shell)?;

        let mut term = Terminal::start(self, binary)?;
        term.await_expected(self.shell.started())?;

        term.user_input(&prompt)?;
        let raw = match op {
            Op::Timeout { timeout } => term.await_timeout(timeout, cache.as_deref())?,
        };

        if cache.is_none_or(|old| old != raw) {
            save_cached(&binary.name, &self.prompt, &self.shell, &raw)?;
        }

        let mut actual = String::new();
        for line in term.screen().contents().lines() {
            actual.push_str(line.trim_end());
            actual.push('\n');
        }
        actual.truncate(actual.trim_end().len());
        let matches = self.expected == actual;
        self.stage = if matches {
            Stage::Matches
        } else {
            Stage::Mismatch { actual }
        };
        Ok(matches)
    }
}

impl Chunk {
    fn as_snippet(&self) -> Option<&Snippet> {
        match self {
            Chunk::Text(_) => None,
            Chunk::Chunk(snippet) => Some(snippet),
        }
    }

    fn as_snippet_mut(&mut self) -> Option<&mut Snippet> {
        match self {
            Chunk::Text(_) => None,
            Chunk::Chunk(snippet) => Some(snippet),
        }
    }
}

impl Md {
    pub fn snippets(&self) -> impl Iterator<Item = &Snippet> {
        self.chunks
            .iter()
            .filter_map(Chunk::as_snippet)
            .enumerate()
            .filter_map(|(ix, snippet)| {
                self.active
                    .as_ref()
                    .is_none_or(|a| a.contains(&ix))
                    .then_some(snippet)
            })
    }

    pub fn snippets_mut(&mut self) -> impl Iterator<Item = &mut Snippet> {
        self.chunks
            .iter_mut()
            .filter_map(Chunk::as_snippet_mut)
            .enumerate()
            .filter_map(|(ix, snippet)| {
                self.active
                    .as_ref()
                    .is_none_or(|a| a.contains(&ix))
                    .then_some(snippet)
            })
    }
}
