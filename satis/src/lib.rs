use anyhow::Context;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fmt::Write,
    path::PathBuf,
    process::Command,
    time::Duration,
};
use tempdir::TempDir;
pub use term::Session;

mod config;
mod term;
pub use config::{Binary, Shell, config_from_cargo};

#[derive(Debug, Clone)]
pub struct FileOp {
    pub file: PathBuf,
    pub indices: Option<Vec<usize>>,
}

/// File operations
///
/// Parses a list of files, each file can be followed by zero or more indices
pub fn parse_file_op() -> impl bpaf::Parser<Vec<FileOp>> {
    fn parse_file_set(xs: Vec<std::ffi::OsString>) -> Result<Vec<FileOp>, String> {
        let mut res = Vec::new();
        let mut ix = 0;
        let mut indices = Vec::new();

        while let Some(file) = xs.get(ix) {
            let file = PathBuf::from(file);
            if file.extension().is_none_or(|e| e != "md") {
                return Err(format!("Expected an .md file, got {file:?}"));
            }
            while let Some(value) = xs
                .get(ix + 1)
                .and_then(|v| v.to_str())
                .and_then(|v| v.parse::<usize>().ok())
            {
                indices.push(value);
                ix += 1;
            }

            ix += 1;
            res.push(FileOp {
                file,
                indices: (!indices.is_empty()).then_some(std::mem::take(&mut indices)),
            });
        }

        if res.is_empty() {
            return Err("You need to specify at least one file".to_string());
        }
        Ok(res)
    }
    use bpaf::*;
    positional::<std::ffi::OsString>("FILE/IX")
        .help("One or more files, each file can be followed by zero or more indices")
        .many()
        .parse(parse_file_set)
}

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

    fn cache_key(app: &str, prompt: &str, shell: &Shell, reuse: bool) -> u64 {
        let mut hasher = DefaultHasher::new();
        app.hash(&mut hasher);
        prompt.hash(&mut hasher);
        shell.hash(&mut hasher);
        reuse.hash(&mut hasher);
        hasher.finish()
    }

    fn cache_path(app: &str, prompt: &str, shell: &Shell, reuse: bool) -> PathBuf {
        let mut path = cache_dir();
        path.push(format!("{:x}.bin", cache_key(app, prompt, shell, reuse)));
        path
    }

    pub(crate) fn load_cached(
        app: &str,
        prompt: &str,
        shell: &Shell,
        reuse: bool,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let path = cache_path(app, prompt, shell, reuse);
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
        reuse: bool,
    ) -> anyhow::Result<()> {
        let path = cache_path(app, prompt, shell, reuse);
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

// EnvSetup holds the prepared tempdir and config files for a (binary, shell) pair.
// Can be reused to spawn multiple shell processes or a persistent session.
pub(crate) struct EnvSetup {
    pub(crate) tempdir: TempDir,
    shell: Shell,
    path: OsString,
}

impl EnvSetup {
    /// Build a fresh Command pointing to this environment.
    pub(crate) fn make_command(&self) -> Command {
        let mut run_shell = Command::new(self.shell);
        run_shell.env_clear();
        run_shell.env("TERM", "xterm");
        run_shell.env("LC_ALL", "en_US.UTF-8");
        run_shell.env("PATH", &self.path);
        run_shell.env("HOME", self.tempdir.as_ref());

        match self.shell {
            Shell::Bash => {
                run_shell.env("PS1", "bash$ ");
            }
            Shell::Zsh => {
                run_shell.env("PS1", "zsh%% ");
            }
            Shell::Fish => {
                run_shell.env("XDG_CONFIG_HOME", self.tempdir.as_ref());
            }
        }
        run_shell
    }
}

pub const BASH_COMP: &str = "/usr/share/bash-completion/bash_completion";
impl Shell {
    /// Create the environment: tempdir, fetch completion script, write config files.
    /// Returns EnvSetup that can spawn multiple shell processes from the same env.
    pub(crate) fn prepare_env(self, binary: &Binary) -> anyhow::Result<EnvSetup> {
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
            }
            Shell::Fish => {
                use std::io::Write;
                let fish_conf = home.join("fish");
                let comp_dir = fish_conf.join("completions");
                std::fs::create_dir_all(&comp_dir)?;

                let mut cfg = std::fs::File::create(fish_conf.join("config.fish"))?;
                writeln!(cfg, "fish_config theme choose None")?;
                writeln!(cfg, "set -U fish_greeting \"\"")?;
                writeln!(cfg, "function fish_title\nend")?;
                writeln!(cfg, "function fish_prompt\n    printf 'fish> '\nend")?;

                let app = &binary.name;
                let mut comp = std::fs::File::create(comp_dir.join(format!("{app}.fish")))?;
                comp.write_all(&script)?;
            }
        }
        Ok(EnvSetup {
            tempdir,
            shell: self,
            path,
        })
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
    /// Last paragraph of the preceding text block is a comment
    pub comment: Option<String>,
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
            comment: None,
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

    /// Get the shell for this snippet.
    pub fn shell(&self) -> Shell {
        self.shell
    }

    pub fn is_mismatch(&self) -> bool {
        matches!(self.stage, Stage::Mismatch { .. })
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn is_execution(&self) -> bool {
        !self.prompt.contains("<TAB>")
    }

    pub fn output(&self) -> &str {
        if let Stage::Mismatch { actual } = &self.stage {
            actual.as_str()
        } else {
            &self.expected
        }
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
            comment: _,
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
    pub path: PathBuf,
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
    pub fn open(file: &FileOp) -> anyhow::Result<Self> {
        let path = &file.file;

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
                    comment: _,
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

    pub fn changed(&self) -> bool {
        self.chunks.iter().any(|c| match c {
            Chunk::Text(_) => false,
            Chunk::Chunk(snippet) => matches!(snippet.stage, Stage::Mismatch { .. }),
        })
    }

    /// Last separate line from the preceding text block is a comment
    ///
    /// Useful for debugging with -v
    pub fn populate_comments(&mut self) {
        let mut comment: Option<String> = None;
        for chunk in &mut self.chunks {
            match chunk {
                Chunk::Text(t) => {
                    let trimmed = t.trim();
                    let trimmed = trimmed.rsplit_once("\n\n").map_or(trimmed, |(_, c)| c);
                    comment = if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    };
                }
                Chunk::Chunk(snippet) => {
                    snippet.comment = comment.take();
                }
            }
        }
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
            Shell::Fish => "fish> ",
        }
    }

    fn complete_rev(self) -> usize {
        match self {
            Shell::Bash => 10,
            Shell::Zsh => 7,
            Shell::Fish => 9,
        }
    }
}

impl Snippet {
    pub fn check(&mut self, binary: &Binary, timeout: Duration) -> anyhow::Result<bool> {
        if self.prompt.contains("<TAB>") {
            let mut session = Session::new(self, binary)?;
            session.check_snippet(self, timeout)
        } else {
            self.run_execution(binary)
        }
    }

    /// Run the snippet as a plain execution test.
    pub fn run_execution(&mut self, binary: &Binary) -> anyhow::Result<bool> {
        let mut cmd = Command::new(&binary.name);
        cmd.env("PATH", &binary.path);

        let cmd_line = &self.prompt.trim();
        let args: Vec<&str> = cmd_line.split_whitespace().collect();
        cmd.args(&args[1..]);

        let output = cmd
            .output()
            .with_context(|| format!("running {:?} with args {:?}", binary.name, args))?;

        let mut raw = output.stdout;
        raw.extend_from_slice(&output.stderr);

        let mut actual = String::new();
        for line in String::from_utf8_lossy(&raw).lines() {
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

    pub fn run_raw(&self, binary: &Binary) -> anyhow::Result<String> {
        let mut cmd = Command::new(&binary.name);
        cmd.env("PATH", &binary.path);

        let mut cmd_line = self.prompt.as_str();
        while let Some(stripped) = cmd_line.strip_suffix("<TAB>") {
            cmd_line = stripped;
        }

        if self.prompt.contains("<TAB>") {
            cmd.env("BPAF_COMPLETE_REV", self.shell.complete_rev().to_string());
        }

        let args: Vec<&str> = cmd_line.split_whitespace().collect();
        cmd.args(&args[1..]);
        if cmd_line.ends_with(' ') {
            cmd.arg("");
        }

        let output = cmd
            .output()
            .with_context(|| format!("running {:?} with args {:?}", binary.name, args))?;

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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

    pub fn save(&mut self) -> anyhow::Result<()> {
        let mut out = String::new();
        for chunk in &self.chunks {
            match chunk {
                Chunk::Text(t) => out.push_str(t),
                Chunk::Chunk(snippet) => {
                    let expected = match &snippet.stage {
                        Stage::Mismatch { actual } => actual.as_str(),
                        _ => snippet.expected.as_str(),
                    };
                    writeln!(out, "```console")?;
                    writeln!(out, "{} {}", snippet.shell, snippet.prompt)?;
                    writeln!(out, "{expected}\n```")?;
                }
            }
        }
        std::fs::write(&self.path, out)
            .with_context(|| format!("Writing changes to {:?}", self.path))
    }
}
