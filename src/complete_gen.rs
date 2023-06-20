// completion:
// static: flag names, command names
// dynamic: argument values, positional item values
//
// for static when running collect any parser that fails
//
// OR: combine completions
// AND: usual logic without shortcircuits
//
// for static completion it's enough to collect items
// for argument completion - only one argument(Comp::Meta) should be active at once
//
// for rendering prefer longer version of names
//
// complete short names to long names if possible

use crate::{
    args::{Arg, State},
    item::ShortLong,
    parsers::NamedArg,
    Doc, ShellComp,
};
use std::ffi::OsStr;

#[derive(Clone, Debug)]
pub(crate) struct Complete {
    /// completions accumulated so far
    comps: Vec<Comp>,
    pub(crate) output_rev: usize,

    /// don't try to suggest any more positional items after there's a positional item failure
    /// or parsing in progress
    pub(crate) no_pos_ahead: bool,
}

impl Complete {
    pub(crate) fn new(output_rev: usize) -> Self {
        Self {
            comps: Vec::new(),
            output_rev,
            no_pos_ahead: false,
        }
    }
}

impl State {
    /// Add a new completion hint for flag, if needed
    pub(crate) fn push_flag(&mut self, named: &NamedArg) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Flag {
                extra: CompExtra {
                    depth,
                    group: None,
                    help: named.help.as_ref().and_then(Doc::to_completion),
                },
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for an argument, if needed
    pub(crate) fn push_argument(&mut self, named: &NamedArg, metavar: &'static str) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Argument {
                extra: CompExtra {
                    depth,
                    group: None,
                    help: named.help.as_ref().and_then(Doc::to_completion),
                },
                metavar,
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for metadata, if needed
    ///
    /// is_meta is set to true when we are trying to parse the value and false if
    /// when meta
    pub(crate) fn push_metavar(
        &mut self,
        meta: &'static str,
        help: &Option<Doc>,
        is_argument: bool,
    ) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            let extra = CompExtra {
                depth,
                group: None,
                help: help.as_ref().and_then(Doc::to_completion),
            };

            comp.comps.push(Comp::Metavariable {
                extra,
                meta,
                is_argument,
            })
        }
    }

    /// Add a new completion hint for command, if needed
    pub(crate) fn push_command(
        &mut self,
        name: &'static str,
        short: Option<char>,
        help: &Option<Doc>,
    ) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Command {
                extra: CompExtra {
                    depth,
                    group: None,
                    help: help.as_ref().and_then(Doc::to_completion),
                },
                name,
                short,
            });
        }
    }

    /// Clear collected completions if enabled
    pub(crate) fn clear_comps(&mut self) {
        if let Some(comp) = self.comp_mut() {
            comp.comps.clear();
        }
    }

    /// Insert a literal value with some description for completion
    ///
    /// In practice it's "--"
    pub(crate) fn push_pos_sep(&mut self) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Value {
                extra: CompExtra {
                    depth,
                    group: None,
                    help: Some("Positional only items after this token".to_owned()),
                },
                body: "--".to_owned(),
                is_argument: false,
            });
        }
    }

    /// Insert a bunch of items
    pub(crate) fn push_with_group(&mut self, group: Option<Doc>, comps: &mut Vec<Comp>) {
        let group = group.map(|g| g.monochrome(true));
        if let Some(comp) = self.comp_mut() {
            for mut item in comps.drain(..) {
                if let Some(group) = group.as_ref() {
                    item.set_group(group.clone());
                }
                comp.comps.push(item);
            }
        }
    }
}
/*
impl Arg {
    pub(crate) fn is_word(&self) -> bool {
        match self {
            Arg::Short(..) | Arg::Long(..) => false,
            Arg::Word(_) | Arg::PosWord(_) => true,
        }
    }
}*/

impl Complete {
    pub(crate) fn push_shell(&mut self, op: ShellComp, depth: usize) {
        self.comps.push(Comp::Shell {
            extra: CompExtra {
                depth,
                group: None,
                help: None,
            },
            script: op,
        });
    }

    pub(crate) fn push_value(
        &mut self,
        body: String,
        help: Option<String>,
        depth: usize,
        is_argument: bool,
    ) {
        self.comps.push(Comp::Value {
            body,
            is_argument,
            extra: CompExtra {
                depth,
                group: None,
                help,
            },
        });
    }

    pub(crate) fn push_comp(&mut self, comp: Comp) {
        self.comps.push(comp);
    }

    pub(crate) fn extend_comps(&mut self, comps: Vec<Comp>) {
        self.comps.extend(comps);
    }

    pub(crate) fn drain_comps(&mut self) -> std::vec::Drain<Comp> {
        self.comps.drain(0..)
    }

    pub(crate) fn swap_comps(&mut self, other: &mut Vec<Comp>) {
        std::mem::swap(other, &mut self.comps);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompExtra {
    /// Used by complete_gen to separate commands from each other
    depth: usize,

    /// Render this option in a group along with all other items with the same name
    group: Option<String>,

    /// help message attached to a completion item
    help: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    /// short or long flag
    Flag {
        extra: CompExtra,
        name: ShortLong,
    },

    /// argument + metadata
    Argument {
        extra: CompExtra,
        name: ShortLong,
        metavar: &'static str,
    },

    ///
    Command {
        extra: CompExtra,
        name: &'static str,
        short: Option<char>,
    },

    /// comes from completed values, part of "dynamic" completion
    Value {
        extra: CompExtra,
        body: String,
        /// values from arguments (say -p=SPEC and user already typed "-p b"
        /// should suppress all other options except for metavaraiables?
        ///
        is_argument: bool,
    },

    Metavariable {
        extra: CompExtra,
        meta: &'static str,
        is_argument: bool,
    },

    Shell {
        extra: CompExtra,
        script: ShellComp,
    },
}

impl Comp {
    /// to avoid leaking items with higher depth into items with lower depth
    fn depth(&self) -> usize {
        match self {
            Comp::Command { extra, .. }
            | Comp::Value { extra, .. }
            | Comp::Flag { extra, .. }
            | Comp::Shell { extra, .. }
            | Comp::Metavariable { extra, .. }
            | Comp::Argument { extra, .. } => extra.depth,
        }
    }

    /// completer needs to replace meta placeholder with actual values - uses this
    ///
    /// value indicates if it's an argument or a positional meta
    pub(crate) fn is_metavar(&self) -> Option<bool> {
        if let Comp::Metavariable { is_argument, .. } = self {
            Some(*is_argument)
        } else {
            None
        }
    }

    pub(crate) fn set_group(&mut self, group: String) {
        let extra = match self {
            Comp::Flag { extra, .. }
            | Comp::Argument { extra, .. }
            | Comp::Command { extra, .. }
            | Comp::Value { extra, .. }
            | Comp::Shell { extra, .. }
            | Comp::Metavariable { extra, .. } => extra,
        };
        if extra.group.is_none() {
            extra.group = Some(group);
        }
    }
}

#[derive(Debug)]
struct ShowComp<'a> {
    /// value to be actually inserted by the autocomplete system
    subst: String,

    /// pretty rendering which might include metavars, etc
    pretty: String,

    extra: &'a CompExtra,
}

impl Arg {
    fn and_os_string(&self) -> Option<(&Self, &OsStr)> {
        match self {
            Arg::Short(_, _, s) => {
                if s.is_empty() {
                    None
                } else {
                    Some((self, s))
                }
            }
            Arg::Long(_, _, s) | Arg::ArgWord(s) | Arg::Word(s) | Arg::PosWord(s) => {
                Some((self, s))
            }
        }
    }
}

fn pair_to_os_string<'a>(pair: (&'a Arg, &'a OsStr)) -> Option<(&'a Arg, &'a str)> {
    Some((pair.0, pair.1.to_str()?))
}

impl State {
    /// Generate completion from collected heads
    ///
    /// before calling this method we run parser in "complete" mode and collect live heads inside
    /// `self.comp`, this part goes over collected heads and generates possible completions from
    /// that
    pub(crate) fn check_complete(&self) -> Option<String> {
        let comp = self.comp_ref()?;

        let mut items = self
            .items
            .iter()
            .rev()
            .filter_map(Arg::and_os_string)
            .filter_map(pair_to_os_string);

        // try get a current item to complete - must be non-virtual right most one
        // value must be present here, and can fail only for non-utf8 values
        // can't do much completing with non-utf8 values since bpaf needs to print them to stdout
        let (_, lit) = items.next()?;

        // For cases like "-k=val", "-kval", "--key=val", "--key val"
        // last value is going  to be either Arg::Word or Arg::ArgWord
        // so to perform full completion we look at the preceeding item
        // and use it's value if it was a composite short/long argument
        let (pos_only, full_lit) = match items.next() {
            Some((Arg::Short(_, true, _os) | Arg::Long(_, true, _os), full_lit)) => {
                (false, full_lit)
            }
            Some((Arg::PosWord(_), _)) => (true, lit),
            _ => (false, lit),
        };

        //        let pos_only = matches!(arg, Arg::PosWord(_));
        //        let is_word = matches!(arg, Arg::PosWord(_) | Arg::Word(_));

        let (items, shell) = comp.complete(lit, pos_only);

        Some(match comp.output_rev {

            0 => render_test(&items, &shell, full_lit),
            7 => render_zsh(&items, &shell, full_lit),
            8 => render_bash(&items, &shell),
            unk => panic!("Unsupported output revision {}, you need to genenerate your shell completion files for the app", unk)
        }.unwrap())
    }
}

/// Try to expand short string names into long names if possible
fn preferred_name(name: ShortLong) -> String {
    match name {
        ShortLong::Short(s) => format!("-{}", s),
        ShortLong::Long(l) | ShortLong::ShortLong(_, l) => format!("--{}", l),
    }
}

// check if argument can possibly match the argument passed in and returns a preferrable replacement
fn arg_matches(arg: &str, name: ShortLong) -> Option<String> {
    // "" and "-" match any flag
    if arg.is_empty() || arg == "-" {
        return Some(preferred_name(name));
    }

    let mut can_match = false;

    // separately check for short and long names, fancy strip prefix things is here to avoid
    // allocations and cloning
    match name {
        ShortLong::Long(_) => {}
        ShortLong::Short(s) | ShortLong::ShortLong(s, _) => {
            can_match |= arg
                .strip_prefix('-')
                .and_then(|a| a.strip_prefix(s))
                .map_or(false, str::is_empty);
        }
    }

    // and long string too
    match name {
        ShortLong::Short(_) => {}
        ShortLong::Long(l) | ShortLong::ShortLong(_, l) => {
            can_match |= arg.strip_prefix("--").map_or(false, |s| l.starts_with(s));
        }
    }

    if can_match {
        Some(preferred_name(name))
    } else {
        None
    }
}
fn cmd_matches(arg: &str, name: &'static str, short: Option<char>) -> Option<&'static str> {
    // partial long name and exact short name match anything
    if name.starts_with(arg)
        || short.map_or(false, |s| {
            // avoid allocations
            arg.strip_prefix(s).map_or(false, str::is_empty)
        })
    {
        Some(name)
    } else {
        None
    }
}

impl Comp {
    /// this completion should suppress anything else that is not a value
    fn only_value(&self) -> bool {
        match self {
            Comp::Flag { .. } | Comp::Argument { .. } | Comp::Command { .. } => false,
            Comp::Metavariable { is_argument, .. } | Comp::Value { is_argument, .. } => {
                *is_argument
            }
            Comp::Shell { .. } => true,
        }
    }
    fn is_pos(&self) -> bool {
        match self {
            Comp::Flag { .. } | Comp::Argument { .. } | Comp::Command { .. } => false,
            Comp::Value { is_argument, .. } => !is_argument,
            Comp::Metavariable { .. } | Comp::Shell { .. } => true,
        }
    }
}

impl Complete {
    fn complete(&self, arg: &str, pos_only: bool) -> (Vec<ShowComp>, Vec<ShellComp>) {
        let mut items: Vec<ShowComp> = Vec::new();
        let mut shell = Vec::new();
        let max_depth = self.comps.iter().map(Comp::depth).max().unwrap_or(0);
        let mut only_values = false;

        for item in self
            .comps
            .iter()
            .filter(|c| c.depth() == max_depth && (!pos_only || c.is_pos()))
        {
            match (only_values, item.only_value()) {
                (true, true) | (false, false) => {}
                (true, false) => continue,
                (false, true) => {
                    only_values = true;
                    items.clear();
                }
            }

            match item {
                Comp::Command { name, short, extra } => {
                    if let Some(long) = cmd_matches(arg, name, *short) {
                        items.push(ShowComp {
                            subst: long.to_string(),
                            pretty: long.to_string(),
                            extra,
                        });
                    }
                }

                Comp::Flag { name, extra } => {
                    if let Some(long) = arg_matches(arg, *name) {
                        items.push(ShowComp {
                            pretty: long.clone(),
                            subst: long,
                            extra,
                        });
                    }
                }

                Comp::Argument {
                    name,
                    metavar,
                    extra,
                } => {
                    if let Some(long) = arg_matches(arg, *name) {
                        items.push(ShowComp {
                            pretty: format!("{}={}", long, metavar),
                            subst: long,
                            extra,
                        });
                    }
                }

                Comp::Value {
                    body,
                    extra,
                    is_argument: _,
                } => {
                    items.push(ShowComp {
                        pretty: body.clone(),
                        extra,
                        subst: body.clone(),
                    });
                }

                Comp::Metavariable {
                    extra,
                    meta,
                    is_argument,
                } => {
                    if !is_argument && !pos_only && arg.starts_with('-') {
                        continue;
                    }
                    items.push(ShowComp {
                        extra,
                        pretty: meta.to_string(),
                        subst: String::new(),
                    });
                }

                Comp::Shell { script, .. } => {
                    shell.push(*script);
                }
            }
        }

        (items, shell)
    }
}

impl std::fmt::Display for ShowComp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(help) = &self.extra.help {
            write!(f, "{:24} -- {}", self.pretty, help)
        } else {
            write!(f, "{}", self.pretty)
        }
    }
}

fn render_zsh(
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

fn render_bash(items: &[ShowComp], ops: &[ShellComp]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();

    // Bash is strange when it comes to completion - rather than taking
    // a glob - _filedir takes an extension which it later to include uppercase
    // version as well and to include "*." in front. For compatibility with
    // zsh and other shells - this code strips "*." from the beginning....
    fn bashmask(i: &str) -> &str {
        i.strip_prefix("*.").unwrap_or(i)
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
            writeln!(res, "COMPREPLY+=({:?} '')", items[0].pretty)?;
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

fn render_test(
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
