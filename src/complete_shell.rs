use crate::{complete_gen::ShowComp, Error, Meta, Parser, State};

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
                if ci.is_metavar().is_some() {
                    comp.push_shell(self.op, depth);
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
            ShellComp::File { mask: Some(mask) } => writeln!(res, "_files -g '{}'", mask),
            ShellComp::Dir { mask: None } => writeln!(res, "_files -/"),
            ShellComp::Dir { mask: Some(mask) } => writeln!(res, "_files -/ -g '{}'", mask),
            ShellComp::Raw { zsh, .. } => writeln!(res, "{}", zsh),
            ShellComp::Nothing => Ok(()),
        }?
    }

    if items.len() == 1 {
        if items[0].subst.is_empty() {
            writeln!(res, "compadd -- {:?}", items[0].pretty)?;
            writeln!(res, "compadd ''")?;
            return Ok(res);
        } else {
            return Ok(format!("compadd -- {:?}\n", items[0].subst));
        }
    }
    writeln!(res, "local -a descr")?;

    for item in items {
        writeln!(res, "descr=(\"{}\")", item)?;
        //        writeln!(res, "args=(\"{}\")", item.subst)?;
        if let Some(group) = &item.extra.group {
            writeln!(
                res,
                "compadd -d descr -V {:?} -X {:?} -- {:?}",
                group, group, item.subst,
            )?;
        } else {
            writeln!(res, "compadd -d descr -- {:?}", item.subst)?;
        }
    }
    Ok(res)
}

pub(crate) fn render_bash(
    items: &[ShowComp],
    ops: &[ShellComp],
    full_lit: &str,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();

    // Bash is strange when it comes to completion - rather than taking
    // a glob - _filedir takes an extension which it later to include uppercase
    // version as well and to include "*." in front. For compatibility with
    // zsh and other shells - this code strips "*." from the beginning....
    fn bashmask(i: &str) -> &str {
        i.strip_prefix("*.").unwrap_or(i)
    }

    if items.is_empty() && ops.is_empty() {
        return Ok(format!("COMPREPLY += ( {:?})\n", full_lit));
    }

    for op in ops {
        match op {
            ShellComp::File { mask: None } => write!(res, "_filedir"),
            ShellComp::File { mask: Some(mask) } => {
                writeln!(res, "_filedir '{}'", bashmask(mask))
            }
            ShellComp::Dir { mask: None } => write!(res, "_filedir -d"),
            ShellComp::Dir { mask: Some(mask) } => {
                writeln!(res, "_filedir -d '{}'", bashmask(mask))
            }
            ShellComp::Raw { bash, .. } => writeln!(res, "{}", bash),
            ShellComp::Nothing => Ok(()),
        }?;
    }

    if items.len() == 1 {
        if items[0].subst.is_empty() {
            writeln!(res, "COMPREPLY+=( {:?} '')", items[0].pretty)?;
        } else {
            writeln!(res, "COMPREPLY+=( {:?} )\n", items[0].subst)?;
        }

        return Ok(res);
    }
    let mut prev = "";
    for item in items.iter() {
        if let Some(group) = &item.extra.group {
            if prev != group {
                prev = group;
                writeln!(res, "COMPREPLY+=({:?})", group)?;
            }
        }
        writeln!(res, "COMPREPLY+=(\"{}\")", item)?;
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
        writeln!(res, "{:?}", op)?
    }

    Ok(res)
}

pub(crate) fn render_fish(
    items: &[ShowComp],
    ops: &[ShellComp],
    full_lit: &str,
    app: &str,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    if items.is_empty() && ops.is_empty() {
        return Ok(format!("complete -c {} --arguments={}", app, full_lit));
    }
    let shared = if ops.is_empty() { "-f " } else { "" };
    for item in items.iter().rev().filter(|i| !i.subst.is_empty()) {
        write!(res, "complete -c {} {}", app, shared)?;
        if let Some(long) = item.subst.strip_prefix("--") {
            write!(res, "--long-option {} ", long)?;
        } else if let Some(short) = item.subst.strip_prefix('-') {
            write!(res, "--short-option {} ", short)?;
        } else {
            write!(res, "-a {} ", item.subst)?;
        }
        if let Some(help) = item.extra.help.as_deref() {
            write!(res, "-d {:?}", help)?;
        }
        writeln!(res)?;
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
