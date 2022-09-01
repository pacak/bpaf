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

use crate::{args::Arg, item::ShortLong, Args, Error, Named};
use std::ffi::OsStr;

#[derive(Clone, Debug, Default)]
pub(crate) struct Complete {
    /// completions accumulated so far
    pub(crate) comps: Vec<Comp>,
}

impl Args {
    /// Add a new completion hint for flag if completion is enabled
    pub(crate) fn push_flag(&mut self, named: &Named) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Flag {
                name: ShortLong::from(named),
                help: named.help.clone(),
                depth: self.depth,
            })
        }
    }

    /// Add a new completion hint for metadata if completion is enabled
    pub(crate) fn push_metadata(
        &mut self,
        meta: &'static str,
        help: &Option<String>,
        is_arg: bool,
    ) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Meta {
                meta,
                depth: self.depth,
                is_arg,
                help: help.clone(),
            })
        }
    }

    /// Add a new completion hint for command if completion is enabled
    pub(crate) fn push_command(
        &mut self,
        name: &'static str,
        short: Option<char>,
        help: &Option<String>,
    ) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Command {
                name,
                short,
                help: help.clone(),
                depth: self.depth,
            })
        }
    }

    /// Clear collected completions if enabled
    pub(crate) fn clear_comps(&mut self) {
        if let Some(comp) = &mut self.comp {
            comp.comps.clear()
        }
    }

    pub(crate) fn push_value(&mut self, body: &str, help: &Option<String>, is_arg: bool) {
        if let Some(comp) = &mut self.comp {
            comp.comps.push(Comp::Value {
                depth: self.depth,
                body: body.to_owned(),
                help: help.clone(),
                is_arg,
            })
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
            help,
            depth,
            is_arg,
        });
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    /// Flag. corresponds to -f | --f parts of both flags and arguments
    Flag {
        depth: usize,
        name: ShortLong,
        help: Option<String>,
    },

    Command {
        depth: usize,
        name: &'static str,
        short: Option<char>,
        help: Option<String>,
    },

    /// comes from completed values, part of "dynamic" completion
    Value {
        depth: usize,
        body: String,
        help: Option<String>,
        is_arg: bool,
    },

    /// Placeholder completion - static completion
    Meta {
        depth: usize,
        meta: &'static str,
        /// true for argument metas
        /// false for positional metas
        is_arg: bool,
        help: Option<String>,
    },
}

impl Comp {
    /// depth is used to track where this parser is relative to other parser, mostly
    /// to prevent completion from commands from leaking into levels below them
    fn depth(&self) -> usize {
        match self {
            Comp::Command { depth, .. }
            | Comp::Value { depth, .. }
            | Comp::Meta { depth, .. }
            | Comp::Flag { depth, .. } => *depth,
        }
    }

    /// completer needs to replace meta placeholder with actual values - uses this
    ///
    /// value indicates if it's an argument or a positional meta
    pub(crate) fn meta_type(&self) -> Option<bool> {
        match self {
            Comp::Command { .. } | Comp::Value { .. } | Comp::Flag { .. } => None,
            Comp::Meta { is_arg, .. } => Some(*is_arg),
        }
    }
}

#[derive(Debug)]
struct ShowComp<'a> {
    /// completion description, only rendered if there's several of them
    descr: &'a Option<String>,

    /// substitutions to use
    subst: String,

    /// if we start rendering values - drop everything else and finish them.
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
        // are we even active?
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

        // try to get an item we are completing - must be non-virtual right most one
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
    fn complete(
        &self,
        arg: &str,
        is_word: bool,
        pos_only: bool,
    ) -> Result<String, std::fmt::Error> {
        let mut items: Vec<ShowComp> = Vec::new();
        let max_depth = self.comps.iter().map(Comp::depth).max().unwrap_or(0);
        let mut has_values = false;

        let mut metas = std::collections::BTreeSet::new();

        for item in self.comps.iter().filter(|c| c.depth() == max_depth) {
            match item {
                Comp::Command {
                    name,
                    short,
                    help,
                    depth: _,
                } => {
                    if pos_only {
                        continue;
                    }
                    if let Some(long) = cmd_matches(arg, name, *short) {
                        items.push(ShowComp {
                            subst: long.to_string(),
                            descr: help,
                            is_value: false,
                        });
                    }
                }

                Comp::Flag {
                    name,
                    help,
                    depth: _,
                } => {
                    if pos_only {
                        continue;
                    }
                    if let Some(long) = arg_matches(arg, *name) {
                        items.push(ShowComp {
                            subst: long,
                            descr: help,
                            is_value: false,
                        });
                    }
                }
                Comp::Value {
                    body,
                    help,
                    depth: _,
                    is_arg,
                } => {
                    has_values |= is_arg;
                    items.push(ShowComp {
                        descr: help,
                        subst: body.clone(),
                        is_value: true,
                    });
                }
                Comp::Meta {
                    meta,
                    depth: _,
                    is_arg,
                    help,
                } => {
                    // only words can go in place of meta, not ags/flags
                    if !is_word {
                        continue;
                    }

                    // deduplicate metadata - in case we are dealing with many positionals, etc.
                    if !metas.insert(meta) {
                        continue;
                    }

                    // if all we have is metadata - preserve original user input
                    let mut subst = if arg.is_empty() {
                        format!("<{}>", meta)
                    } else {
                        arg.to_string()
                    };

                    // suppress all other completion when trying to complete argument's meta:
                    // if valid arguments are `-a <A> | -b <B>` and we see `-a` - suggesting
                    // user to type `-b` would be wrong
                    if *is_arg {
                        subst.push('\n');
                        return Ok(subst);
                    }
                    items.push(ShowComp {
                        descr: help,
                        subst,
                        is_value: false,
                    });
                }
            }
        }

        // similar to handling metadata from the case above but now we are
        // actually completing values for A
        if has_values {
            items.retain(|i| i.is_value);
        }
        self.render(&items)
    }

    fn render(&self, items: &[ShowComp]) -> Result<String, std::fmt::Error> {
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
}

// to allow using ? inside check_complete
impl From<std::fmt::Error> for Error {
    fn from(_: std::fmt::Error) -> Self {
        Error::Stderr("Couldn't render completion info".to_string())
    }
}
