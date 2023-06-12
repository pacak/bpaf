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
    complete_shell::{write_shell, Shell},
    item::ShortLong,
    parsers::NamedArg,
    CompleteDecor, Doc, ShellComp,
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
        if !self.valid_complete_head() {
            return;
        }
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Flag {
                extra: CompExtra {
                    depth,
                    hidden_group: "",
                    visible_group: "",
                    help: named.help.as_ref().and_then(Doc::to_completion),
                },
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for an argument, if needed
    pub(crate) fn push_argument(&mut self, named: &NamedArg, metavar: &'static str) {
        if !self.valid_complete_head() {
            return;
        }
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Argument {
                extra: CompExtra {
                    depth,
                    hidden_group: "",
                    visible_group: "",
                    help: named.help.as_ref().and_then(Doc::to_completion),
                },
                metavar,
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for metadata, if needed
    pub(crate) fn push_metadata(&mut self, meta: &'static str, help: &Option<Doc>, is_arg: bool) {
        if !self.valid_complete_head() {
            return;
        }
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Positional {
                extra: CompExtra {
                    depth,
                    hidden_group: "",
                    visible_group: "",
                    help: help.as_ref().and_then(Doc::to_completion),
                },
                meta,
                is_arg,
            });
        }
    }

    /// Add a new completion hint for command, if needed
    pub(crate) fn push_command(
        &mut self,
        name: &'static str,
        short: Option<char>,
        help: &Option<Doc>,
    ) {
        if !self.valid_complete_head() {
            return;
        }
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Command {
                extra: CompExtra {
                    depth,
                    hidden_group: "",
                    visible_group: "",
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

    pub(crate) fn push_value(&mut self, body: &str, help: &Option<String>, is_arg: bool) {
        if !self.valid_complete_head() {
            return;
        }
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            comp.comps.push(Comp::Value {
                extra: CompExtra {
                    depth,
                    hidden_group: "",
                    visible_group: "",
                    help: help.clone(),
                },
                body: body.to_owned(),
                is_arg,
            });
        }
    }

    pub(crate) fn extend_with_style(&mut self, style: CompleteDecor, comps: &mut Vec<Comp>) {
        if !self.valid_complete_head() {
            return;
        }
        if let Some(comp) = self.comp_mut() {
            for mut item in comps.drain(..) {
                item.set_decor(style);
                comp.comps.push(item);
            }
        }
    }
}
impl Arg {
    pub(crate) fn is_word(&self) -> bool {
        match self {
            Arg::Short(..) | Arg::Long(..) => false,
            Arg::Word(_) | Arg::PosWord(_) => true,
        }
    }
}

impl Complete {
    pub(crate) fn push_shell(&mut self, op: ShellComp, depth: usize, is_arg: bool) {
        self.comps.push(Comp::Shell {
            extra: CompExtra {
                depth,
                hidden_group: "",
                visible_group: "",
                help: None,
            },
            script: op,
            is_arg,
        });
    }

    pub(crate) fn push_value(
        &mut self,
        body: String,
        help: Option<String>,
        depth: usize,
        is_arg: bool,
    ) {
        self.comps.push(Comp::Value {
            body,
            extra: CompExtra {
                depth,
                hidden_group: "",
                visible_group: "",
                help,
            },
            is_arg,
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
    /// used by complete_gen to separate commands from each other
    depth: usize,

    /// hidden group, "" if absent
    hidden_group: &'static str,

    /// visible group, "" if absent
    visible_group: &'static str,

    /// custom help message to render, if present
    help: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    Flag {
        extra: CompExtra,
        name: ShortLong,
    },

    Argument {
        extra: CompExtra,
        name: ShortLong,
        metavar: &'static str,
    },

    Command {
        extra: CompExtra,
        name: &'static str,
        short: Option<char>,
    },

    /// comes from completed values, part of "dynamic" completion
    Value {
        extra: CompExtra,
        body: String,
        is_arg: bool,
    },

    /// Placeholder completion - static completion
    Positional {
        extra: CompExtra,
        meta: &'static str,
        is_arg: bool,
    },
    Shell {
        extra: CompExtra,
        script: ShellComp,
        is_arg: bool,
    },
}

impl Comp {
    /// to avoid leaking items with higher depth into items with lower depth
    fn depth(&self) -> usize {
        match self {
            Comp::Command { extra, .. }
            | Comp::Value { extra, .. }
            | Comp::Positional { extra, .. }
            | Comp::Flag { extra, .. }
            | Comp::Shell { extra, .. }
            | Comp::Argument { extra, .. } => extra.depth,
        }
    }

    /// completer needs to replace meta placeholder with actual values - uses this
    ///
    /// value indicates if it's an argument or a positional meta
    pub(crate) fn meta_type(&self) -> Option<bool> {
        match self {
            Comp::Command { .. }
            | Comp::Value { .. }
            | Comp::Flag { .. }
            | Comp::Shell { .. }
            | Comp::Argument { .. } => None,
            Comp::Positional { is_arg, .. } => Some(*is_arg),
        }
    }

    pub(crate) fn set_decor(&mut self, style: CompleteDecor) {
        let extra = match self {
            Comp::Flag { extra, .. }
            | Comp::Argument { extra, .. }
            | Comp::Command { extra, .. }
            | Comp::Value { extra, .. }
            | Comp::Shell { extra, .. }
            | Comp::Positional { extra, .. } => extra,
        };
        match style {
            CompleteDecor::HiddenGroup(name) => extra.hidden_group = name,
            CompleteDecor::VisibleGroup(name) => extra.visible_group = name,
        }
    }
}

#[derive(Debug)]
struct ShowComp<'a> {
    /// completion description, only rendered if there's several of them
    descr: &'a Option<String>,

    /// substitutions to use
    subst: String,

    /// pretty rendering which might include metavars, etc
    pretty: String,

    extra: &'a CompExtra,

    /// to render only values when values are present
    is_value: bool,
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
            Arg::Long(_, _, s) | Arg::Word(s) | Arg::PosWord(s) => Some((self, s)),
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
        let comp = match self.comp_ref() {
            Some(comp) => comp,
            None => return None,
        };

        let mut items = self
            .items
            .iter()
            .rev()
            .filter_map(Arg::and_os_string)
            .filter_map(pair_to_os_string);

        // try get a current item to complete - must be non-virtual right most one
        // value must be present here, and can fail only for non-utf8 values
        // can't do much completing with non-utf8 values since bpaf needs to print them to stdout
        let (arg, lit) = match items.next() {
            Some(a) => a,
            None => return Some("\n".to_owned()),
        };

        let pos_only = items.clone().any(|i| matches!(i.0, Arg::PosWord(_)));

        let res = comp
            .complete(lit, arg.is_word(), pos_only)
            .expect("format error?");
        Some(res)
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

impl Complete {
    #[allow(clippy::too_many_lines)]
    fn complete(
        &self,
        arg: &str,
        is_word: bool,
        pos_only: bool,
    ) -> Result<String, std::fmt::Error> {
        let mut items: Vec<ShowComp> = Vec::new();
        let mut shell = Vec::new();
        let max_depth = self.comps.iter().map(Comp::depth).max().unwrap_or(0);
        let mut has_values = false;

        for item in self.comps.iter().filter(|c| c.depth() == max_depth) {
            match item {
                Comp::Command { name, short, extra } => {
                    if pos_only {
                        continue;
                    }
                    if let Some(long) = cmd_matches(arg, name, *short) {
                        items.push(ShowComp {
                            subst: long.to_string(),
                            pretty: long.to_string(),
                            descr: &extra.help,
                            is_value: false,
                            extra,
                        });
                    }
                }

                Comp::Flag { name, extra } => {
                    if pos_only {
                        continue;
                    }
                    if let Some(long) = arg_matches(arg, *name) {
                        items.push(ShowComp {
                            pretty: long.clone(),
                            subst: long,
                            descr: &extra.help,
                            is_value: false,
                            extra,
                        });
                    }
                }

                Comp::Argument {
                    name,
                    metavar,
                    extra,
                } => {
                    if pos_only {
                        continue;
                    }
                    if let Some(long) = arg_matches(arg, *name) {
                        items.push(ShowComp {
                            pretty: format!("{} <{}>", long, metavar),
                            subst: long,
                            descr: &extra.help,
                            is_value: false,
                            extra,
                        });
                    }
                }

                Comp::Value {
                    body,
                    extra,
                    is_arg,
                } => {
                    has_values |= is_arg;
                    items.push(ShowComp {
                        pretty: body.clone(),
                        descr: &extra.help,
                        extra,
                        subst: body.clone(),
                        is_value: true,
                    });
                }
                Comp::Positional {
                    meta,
                    is_arg,
                    extra,
                } => {
                    // only words can go in place of meta, not ags/flags
                    if !is_word {
                        continue;
                    }

                    // render empty positionals as placeholders
                    let mut subst = if arg.is_empty() {
                        format!("<{}>", meta)
                    } else {
                        arg.to_string()
                    };

                    // suppress all other completion when trying to complete argument's meta:
                    // if valid arguments are `-a <A> | -b <B>` and current args are `-a` - suggesting
                    // user to type `-b` would be wrong
                    if *is_arg {
                        subst.push('\n');
                        return Ok(subst);
                    }
                    items.push(ShowComp {
                        extra,
                        pretty: subst.clone(),
                        descr: &extra.help,
                        subst,
                        is_value: false,
                    });
                }
                Comp::Shell { script, is_arg, .. } => {
                    has_values |= is_arg;
                    shell.push(*script);
                }
            }
        }

        if has_values {
            items.retain(|i| i.is_value);
        }
        match self.output_rev {
            1 => {
                assert!(shell.is_empty(), "You need to regenerate your completion scripts");
                render_1(&items)
            }
            2 => {
                assert!(shell.is_empty(), "You need to regenerate your completion scripts");
                render_2(&items)
            }
            3 => render_3456(&items, Shell::Bash, &shell),
            4 => render_3456(&items, Shell::Zsh, &shell),
            5 => render_3456(&items, Shell::Fish,&shell),
            6 => render_3456(&items, Shell::Elvish, &shell),
            unk => panic!("Unsupported output revision {}, you need to genenerate your shell completion files for the app", unk)
        }
    }
}
// eveything but zsh, for single items rende replacement as is, otherwise
// render replacement or tab separated replacement and description
fn render_1(items: &[ShowComp]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    if items.len() == 1 {
        writeln!(res, "{}", items[0].subst)?;
    } else {
        for item in items {
            match item.descr {
                None => {
                    writeln!(res, "{}", item.subst)
                }
                Some(descr) => {
                    writeln!(
                        res,
                        "{}\t{}",
                        item.subst,
                        descr.split('\n').next().unwrap_or("")
                    )
                }
            }?;
        }
    }
    Ok(res)
}

// zsh style, renders one item on a line, \0 separated
// - replacement to use
// - description to display, might contain metavars for example
// - visible group - to display a message
// - hidden group, just to group
fn render_2(items: &[ShowComp]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    for item in items {
        write!(res, "{}\0{}", item.subst, item.pretty)?;
        if let Some(h) = &item.extra.help {
            write!(res, "    {}", h.split('\n').next().unwrap_or(""))?;
        }
        write!(res, "\0{}", item.extra.visible_group)?;
        if !item.extra.visible_group.is_empty() && item.extra.hidden_group.is_empty() {
            writeln!(res, "\0{}", item.extra.visible_group)?;
        } else {
            writeln!(res, "\0{}", item.extra.hidden_group)?;
        }
    }
    Ok(res)
}

fn render_3456(
    items: &[ShowComp],
    shell: Shell,
    ops: &[ShellComp],
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut res = String::new();
    if items.len() == 1 && ops.is_empty() {
        write!(res, "literal\t{}", items[0].subst)?;
        return Ok(res);
    }

    for i in items {
        write!(res, "literal\t{}\tshow\t{}", i.subst, i.pretty)?;
        if let Some(h) = &i.extra.help {
            write!(res, "    {}", h.split('\n').next().unwrap_or(""))?;
        }

        if !i.extra.visible_group.is_empty() {
            write!(res, "\tvis_group\t{}", i.extra.visible_group)?;
        }

        if i.extra.hidden_group.is_empty() {
            if !i.extra.visible_group.is_empty() {
                write!(res, "\thid_group\t{}", i.extra.visible_group)?;
            }
        } else {
            write!(res, "\thid_group\t{}", i.extra.hidden_group)?;
        }
        writeln!(res)?;
    }

    for op in ops {
        write_shell(&mut res, shell, *op)?;
    }

    Ok(res)
}
