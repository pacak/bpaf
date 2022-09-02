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

use crate::{args::Arg, item::ShortLong, Args, CompleteDecor, Error, Named};
use std::ffi::OsStr;

#[derive(Clone, Debug)]
pub(crate) struct Complete {
    /// completions accumulated so far
    pub(crate) comps: Vec<Comp>,
    pub(crate) output_rev: usize,
}

impl Complete {
    pub(crate) fn new(output_rev: usize) -> Self {
        Self {
            comps: Vec::new(),
            output_rev,
        }
    }
}

impl Args {
    /// Add a new completion hint for flag, if needed
    pub(crate) fn push_flag(&mut self, named: &Named) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Flag {
                extra: CompExtra {
                    depth: self.depth,
                    hidden_group: "",
                    visible_group: "",
                    help: named.help.clone(),
                },
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for an argument, if needed
    pub(crate) fn push_argument(&mut self, named: &Named, metavar: &'static str) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Argument {
                extra: CompExtra {
                    depth: self.depth,
                    hidden_group: "",
                    visible_group: "",
                    help: named.help.clone(),
                },
                metavar,
                name: ShortLong::from(named),
            });
        }
    }

    /// Add a new completion hint for metadata, if needed
    pub(crate) fn push_metadata(
        &mut self,
        meta: &'static str,
        help: &Option<String>,
        is_arg: bool,
    ) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Positional {
                extra: CompExtra {
                    depth: self.depth,
                    hidden_group: "",
                    visible_group: "",
                    help: help.clone(),
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
        help: &Option<String>,
    ) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Command {
                extra: CompExtra {
                    depth: self.depth,
                    hidden_group: "",
                    visible_group: "",
                    help: help.clone(),
                },
                name,
                short,
            });
        }
    }

    /// Clear collected completions if enabled
    pub(crate) fn clear_comps(&mut self) {
        if let Some(comp) = &mut self.comp {
            comp.comps.clear();
        }
    }

    pub(crate) fn push_value(&mut self, body: &str, help: &Option<String>, is_arg: bool) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Value {
                extra: CompExtra {
                    depth: self.depth,
                    hidden_group: "",
                    visible_group: "",
                    help: help.clone(),
                },
                body: body.to_owned(),
                is_arg,
            });
        }
    }
}
impl Arg {
    pub(crate) fn is_word(&self) -> bool {
        match self {
            Arg::Short(_, _) | Arg::Long(_, _) => false,
            Arg::Word(_) => true,
        }
    }
}

impl Complete {
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
}

#[derive(Clone, Debug)]
pub(crate) struct CompExtra {
    depth: usize,
    hidden_group: &'static str,
    visible_group: &'static str,
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
}

impl Comp {
    /// to avoid leaking items with higher depth into items with lower depth
    fn depth(&self) -> usize {
        match self {
            Comp::Command { extra, .. }
            | Comp::Value { extra, .. }
            | Comp::Positional { extra, .. }
            | Comp::Flag { extra, .. }
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
            Arg::Short(_, s) => {
                if s.is_empty() {
                    None
                } else {
                    Some((self, s))
                }
            }
            Arg::Long(_, s) => Some((self, s)),
            Arg::Word(w) => Some((self, &w.os)),
        }
    }
}

fn pair_to_os_string<'a>(pair: (&'a Arg, &'a OsStr)) -> Option<(&'a Arg, &'a str)> {
    Some((pair.0, pair.1.to_str()?))
}

impl Args {
    pub(crate) fn check_complete(&self) -> Result<(), Error> {
        let comp = match &self.comp {
            Some(comp) => comp,
            None => return Ok(()),
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
        let (arg, lit) = items
            .next()
            .ok_or_else(|| Error::Stdout("\n".to_string()))?;

        // also if lit is to the _right_ of double dash - it can be positional only - so meta or
        // value

        let pos_only = items.any(|(_arg, lit)| lit == "--");

        if let Arg::Short(..) = arg {
            if lit.chars().count() > 2 {
                // don't bother trying to expand -vvvv for now:
                // -vvv<TAB> => -vvv _
                return Err(Error::Stdout(format!("{}\n", lit)));
            }
        }

        let res = comp.complete(lit, arg.is_word(), pos_only)?;
        Err(Error::Stdout(res))
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
                        pretty: match &extra.help {
                            Some(help) => format!("{}    {}", body, help),
                            None => body.clone(),
                        },
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
            }
        }

        if has_values {
            items.retain(|i| i.is_value);
        }
        match self.output_rev {
            1 => render_1(&items),
            2 => render_2(&items),
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

// to allow using ? inside check_complete
impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Error::Stderr("Couldn't render completion info".to_string())
    }
}
