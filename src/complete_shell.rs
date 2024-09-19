use std::borrow::Cow;

use crate::{complete_gen::ShowComp, Error, Meta, Parser, State};

struct Shell<'a>(&'a str);

impl std::fmt::Display for Shell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char('\'')?;
        for c in self.0.chars() {
            if c == '\'' {
                f.write_str("'\\''")
            } else {
                f.write_char(c)
            }?
        }
        f.write_char('\'')?;
        Ok(())
    }
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
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        // same as with ParseComp the goal is to replace metavars added by inner parser
        // with a completion that would call a bash script.
        // unlike ParseComp we don't care if inner parser succeeds

        // stash old completions
        let mut comp_items = Vec::new();
        args.swap_comps_with(&mut comp_items);

        let res = self.inner.eval(args);

        // at this point comp_items contains values added by the inner parser
        args.swap_comps_with(&mut comp_items);

        let depth = args.depth();
        if let Some(comp) = args.comp_mut() {
            for ci in comp_items {
                if let Some(is_argument) = ci.is_metavar() {
                    comp.push_shell(self.op, is_argument, depth);
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

pub(crate) fn render_zsh(
    items: &[ShowComp],
    ops: &[ShellComp],
    full_lit: &str,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();

    if items.is_empty() && ops.is_empty() {
        return Ok(format!("compadd -- {}\n", full_lit));
    }

    for op in ops {
        match op {
            ShellComp::File { mask: None } => writeln!(res, "_files"),
            ShellComp::File { mask: Some(mask) } => writeln!(res, "_files -g {}", Shell(mask)),
            ShellComp::Dir { mask: None } => writeln!(res, "_files -/"),
            ShellComp::Dir { mask: Some(mask) } => writeln!(res, "_files -/ -g {}", Shell(mask)),
            ShellComp::Raw { zsh, .. } => writeln!(res, "{}", zsh),
            ShellComp::Nothing => Ok(()),
        }?;
    }

    if items.len() == 1 {
        if items[0].subst.is_empty() {
            writeln!(res, "compadd -- {}", Shell(items[0].pretty.as_str()))?;
            writeln!(res, "compadd ''")?;
            return Ok(res);
        } else {
            return Ok(format!("compadd -- {}\n", Shell(items[0].subst.as_str())));
        }
    }
    writeln!(res, "local -a descr")?;

    for item in items {
        writeln!(res, "descr=({})", Shell(&item.to_string()))?;
        //        writeln!(res, "args=(\"{}\")", item.subst)?;
        if let Some(group) = &item.extra.group {
            writeln!(
                res,
                "compadd -l -d descr -V {} -X {} -- {}",
                Shell(group),
                Shell(group),
                Shell(&item.subst),
            )?;
        } else {
            // it seems sorting as well as not sorting is done in a group,
            // by default group contains just one element so and `-o nosort`
            // does nothing, while `-V whatever` stops sorting...
            writeln!(
                res,
                "compadd -l -V nosort -d descr -- {}",
                Shell(&item.subst)
            )?;
        }
    }
    Ok(res)
}

pub(crate) fn render_bash(
    items: &[ShowComp],
    ops: &[ShellComp],
    full_lit: &str,
) -> Result<String, std::fmt::Error> {
    // Bash is strange when it comes to completion - rather than taking
    // a glob - _filedir takes an extension which it later to include uppercase
    // version as well and to include "*." in front. For compatibility with
    // zsh and other shells - this code strips "*." from the beginning....
    //
    // Second difference between bash and zsh is that if you are trying to
    // allow for multiple extensions zsh takes a sane "*.(foo|bar|baz)" approach,
    // while bash wants it to be "@(foo|bar|baz)"
    //
    // This doesn't cover all the possible masks, I suspect that the right way of
    // handling this would be ignoring the shell machinery and handling masks on the
    // Rust side... But for now try this
    //
    fn bashmask(i: &str) -> Cow<str> {
        let i = i.strip_prefix("*.").unwrap_or(i);

        if i.starts_with('(') {
            Cow::Owned(format!("@{}", i))
        } else {
            Cow::Borrowed(i)
        }
    }

    use std::fmt::Write;
    let mut res = String::new();

    if items.is_empty() && ops.is_empty() {
        return Ok(format!("COMPREPLY+=({})\n", Shell(full_lit)));
    }

    let init = "local cur prev words cword ; _init_completion || return ;";
    for op in ops {
        match op {
            ShellComp::File { mask: None } => write!(res, "{} _filedir", init),
            ShellComp::File { mask: Some(mask) } => {
                writeln!(res, "{} _filedir {}", init, Shell(&bashmask(mask)))
            }
            ShellComp::Dir { mask: None } => write!(res, "{} _filedir -d", init),
            ShellComp::Dir { mask: Some(mask) } => {
                writeln!(res, "{} _filedir -d {}", init, Shell(&bashmask(mask)))
            }
            ShellComp::Raw { bash, .. } => writeln!(res, "{}", bash),
            ShellComp::Nothing => Ok(()),
        }?;
    }

    if items.len() == 1 {
        if items[0].subst.is_empty() {
            writeln!(res, "COMPREPLY+=( {} '')", Shell(&items[0].pretty))?;
        } else {
            writeln!(res, "COMPREPLY+=( {} )\n", Shell(&items[0].subst))?;
        }

        return Ok(res);
    }
    let mut prev = "";
    for item in items.iter() {
        if let Some(group) = &item.extra.group {
            if prev != group {
                prev = group;
                writeln!(res, "COMPREPLY+=({})", Shell(group))?;
            }
        }
        writeln!(res, "COMPREPLY+=({})", Shell(&item.to_string()))?;
    }

    Ok(res)
}

pub(crate) fn render_test(
    items: &[ShowComp],
    ops: &[ShellComp],
    lit: &str,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;

    if items.is_empty() && ops.is_empty() {
        return Ok(format!("{}\n", lit));
    }

    if items.len() == 1 && ops.is_empty() && !items[0].subst.is_empty() {
        return Ok(items[0].subst.clone());
    }

    let mut res = String::new();
    for op in items {
        writeln!(
            res,
            "{}\t{}\t{}\t{}",
            op.subst,
            op.pretty,
            op.extra.group.as_deref().unwrap_or(""),
            op.extra.help.as_deref().unwrap_or("")
        )?;
    }
    writeln!(res)?;
    for op in ops {
        writeln!(res, "{:?}", op)?;
    }

    Ok(res)
}

pub(crate) fn render_fish(
    items: &[ShowComp],
    ops: &[ShellComp],
    full_lit: &str,
    _app: &str,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    if items.is_empty() && ops.is_empty() {
        writeln!(res, "{}", full_lit)?;
    }

    // skip things without substitutions, I think they
    // are headers and such, and fish is a bit
    for item in items.iter().rev().filter(|i| !i.subst.is_empty()) {
        if let Some(help) = item.extra.help.as_deref() {
            writeln!(res, "{}\t{}", item.subst, help)?;
        } else {
            writeln!(res, "{}", item.subst)?;
        }
    }

    Ok(res)
}

pub(crate) fn render_simple(items: &[ShowComp]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    if items.len() == 1 {
        writeln!(res, "{}", items[0].subst)?;
    } else {
        for item in items {
            if let Some(descr) = item.extra.help.as_deref() {
                writeln!(
                    res,
                    "{}\t{}",
                    item.subst,
                    descr.split('\n').next().unwrap_or("")
                )
            } else {
                writeln!(res, "{}", item.subst)
            }?;
        }
    }
    Ok(res)
}
