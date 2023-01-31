use crate::{Args, Error, Meta, Parser};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
}

#[derive(Debug, Clone, Copy)]
/// Shell specific completion
#[non_exhaustive]
pub enum ShellComp {
    /// A file or directory name with an optional file mask.
    ///
    /// For bash filemask should start with `*.` or contain only the
    /// extension
    File {
        /// Optional filemask to use, no spaces, no tabs
        mask: Option<&'static str>,
    },

    /// Similar to `File` but limited to directories only
    /// For bash filemask should start with `*.` or contain only the
    /// extension
    Dir {
        /// Optional filemask to use, no spaces, no tabs
        mask: Option<&'static str>,
    },

    /// You can also specify a raw value to use for each supported shell
    ///
    /// It is possible to fill in values for shells you don't want to support
    /// with empty strings but the code is not going to work for those shells
    Raw {
        /// This raw string will be used for `bash` shell
        /// <https://www.gnu.org/software/bash/manual/html_node/Command-Line-Editing.html>
        bash: &'static str,

        /// This raw string will be used for `zsh` shell
        /// <https://zsh.sourceforge.io/Doc/Release/Completion-System.html>
        zsh: &'static str,

        /// This raw string will be used for `fish` shell
        /// <https://fishshell.com/docs/current/completions.html>
        fish: &'static str,

        /// This raw string will be used for `elvish` shell
        /// <https://elv.sh/ref/edit.html#completion-api>
        elvish: &'static str,
    },

    /// Don't produce anything at all from this parser - can be useful if you want to compose
    /// bpaf completion with shell completion
    Nothing,
}

/// Parser that inserts static shell completion into bpaf's dynamic shell completion
#[cfg(feature = "autocomplete")]
pub struct ParseCompShell<P> {
    pub(crate) inner: P,
    pub(crate) op: crate::complete_shell::ShellComp,
}

#[cfg(feature = "autocomplete")]
impl<P, T> Parser<T> for ParseCompShell<P>
where
    P: Parser<T> + Sized,
{
    fn eval(&self, args: &mut Args) -> Result<T, Error> {
        // same as with ParseComp the goal is to replace metavars added by inner parser
        // with a completion that would call a bash script.
        // unlike ParseComp we don't care if inner parser succeeds

        // stash old completions
        let mut comp_items = Vec::new();
        args.swap_comps(&mut comp_items);

        let res = self.inner.eval(args);

        // at this point comp_items contains values added by the inner parser
        args.swap_comps(&mut comp_items);

        let depth = args.depth;
        if let Some(comp) = args.comp_mut() {
            for ci in comp_items {
                if let Some(is_arg) = ci.meta_type() {
                    comp.push_shell(self.op, depth, is_arg);
                } else {
                    comp.push_comp(ci);
                }
            }
        }

        res
    }

    fn meta(&self) -> Meta {
        self.inner.meta()
    }
}

pub(crate) fn write_shell(res: &mut String, shell: Shell, comp: ShellComp) -> std::fmt::Result {
    match shell {
        Shell::Bash => write_bash(res, comp),
        Shell::Zsh => write_zsh(res, comp),
        // not supported at the moment
        Shell::Fish | Shell::Elvish => Ok(()),
    }
}

fn write_bash(res: &mut String, comp: ShellComp) -> std::fmt::Result {
    use std::fmt::Write;
    match comp {
        ShellComp::File { mask: None } => write!(res, "bash\t_filedir"),
        ShellComp::File { mask: Some(mask) } => {
            writeln!(res, "bash\t_filedir '{}'", bashmask(mask))
        }
        ShellComp::Dir { mask: None } => write!(res, "bash\t_filedir -d"),
        ShellComp::Dir { mask: Some(mask) } => {
            writeln!(res, "bash\t_filedir -d '{}'", bashmask(mask))
        }
        ShellComp::Raw { bash, .. } => writeln!(res, "bash\t{}", bash),
        ShellComp::Nothing => Ok(()),
    }
}

// Bash is strange when it comes to completion - rather than taking
// a glob - _filedir takes an extension which it later to include uppercase
// version as well and to include "*." in front. For compatibility with
// zsh and other shells - this code strips "*." from the beginning....
fn bashmask(i: &str) -> &str {
    i.strip_prefix("*.").unwrap_or(i)
}

fn write_zsh(res: &mut String, comp: ShellComp) -> std::fmt::Result {
    use std::fmt::Write;
    match comp {
        ShellComp::File { mask: None } => writeln!(res, "zsh\t_files"),
        ShellComp::File { mask: Some(mask) } => writeln!(res, "zsh\t_files -g '{}'", mask),
        ShellComp::Dir { mask: None } => writeln!(res, "zsh\t_files -/"),
        ShellComp::Dir { mask: Some(mask) } => writeln!(res, "zsh\t_files -/ -g '{}'", mask),
        ShellComp::Raw { zsh, .. } => writeln!(res, "zsh\t{}", zsh),
        ShellComp::Nothing => Ok(()),
    }
}
