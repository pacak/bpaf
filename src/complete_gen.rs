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
    args::Arg,
    complete_run::Style,
    item::{Item, ShortLong},
    Args, Error,
};
use std::ffi::OsStr;

#[derive(Clone, Debug)]
pub(crate) struct Complete {
    /// used do decide which version to render, mostly bash vs everything else
    style: Style,
    /// completions accumulated so far
    pub(crate) comps: Vec<Comp>,
}

impl Complete {
    pub(crate) fn new(style: Style) -> Self {
        Self {
            comps: Vec::new(),
            style,
        }
    }

    pub(crate) fn push_item(&mut self, item: Item, depth: usize) {
        self.comps.push(Comp::Item { item, depth });
    }

    pub(crate) fn push_metadata(
        &mut self,
        meta: &'static str,
        help: Option<String>,
        depth: usize,
        arg: bool,
    ) {
        self.comps.push(Comp::Meta {
            meta,
            depth,
            is_arg: arg,
            help,
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
            help,
            depth,
            is_arg,
        });
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    /// comes from named items, part of "static" completion
    Item { item: Item, depth: usize },

    /// comes from completed values, part of "dynamic" completion
    Value {
        body: String,
        help: Option<String>,
        depth: usize,
        is_arg: bool,
    },

    /// Placeholder completion - static completion
    Meta {
        meta: &'static str,
        depth: usize,
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
            Comp::Item { depth, .. } | Comp::Value { depth, .. } | Comp::Meta { depth, .. } => {
                *depth
            }
        }
    }

    /// completer needs to replace meta placeholder with actual values - uses this
    ///
    /// value indicates if it's an argument or a positional meta
    pub(crate) fn meta_type(&self) -> Option<bool> {
        match self {
            Comp::Item { .. } | Comp::Value { .. } => None,
            Comp::Meta { is_arg, .. } => Some(*is_arg),
        }
    }
}

#[derive(Debug)]
struct ShowComp<'a> {
    /// completion description, only rendered if there's several of them
    descr: &'a Option<String>,

    /// substitution to use for multiple items, unlike subst1 includes metavars
    subst: String,

    /// substitution to use for a single item
    subst1: String,

    /// if we start rendering values - drop everything else and finish them.
    ///
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
                Comp::Item { item, depth: _ } => match item {
                    // we don't push those guys, instead this is Comp::Meta which
                    // is shared between positionals and arguments
                    Item::Positional { .. } => {
                        unreachable!("completion for positional item detected")
                    }
                    Item::Command {
                        name,
                        short,
                        help,
                        meta: _,
                    } => {
                        if pos_only {
                            continue;
                        }
                        if let Some(long) = cmd_matches(arg, name, *short) {
                            items.push(ShowComp {
                                subst: long.to_string(),
                                subst1: long.to_string(),
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                    Item::Flag { name, help } => {
                        if pos_only {
                            continue;
                        }
                        if let Some(long) = arg_matches(arg, *name) {
                            items.push(ShowComp {
                                subst: long.clone(),
                                subst1: long,
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                    Item::Argument {
                        name,
                        metavar,
                        env: _,
                        help,
                    } => {
                        if pos_only {
                            continue;
                        }
                        if let Some(long) = arg_matches(arg, *name) {
                            items.push(ShowComp {
                                subst: format!("{} <{}>", long, metavar),
                                subst1: long,
                                descr: help,
                                is_value: false,
                            });
                        }
                    }
                },
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
                        subst1: body.clone(),
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
                        subst1: subst.clone(),
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
            writeln!(res, "{}", items[0].subst1)?;
        } else {
            // we can probably make this logic simplier by moving
            // rendering to bash side... Maybe one day.
            let max_width = items
                .iter()
                .map(|s| s.subst.chars().count())
                .max()
                .unwrap_or(0);

            for item in items {
                match (self.style, item.descr) {
                    (Style::Bash, None) => writeln!(res, "{}", item.subst),
                    (Style::Bash, Some(descr)) => writeln!(
                        res,
                        "{:padding$}  {}",
                        item.subst,
                        descr,
                        padding = max_width
                    ),
                    (Style::Zsh | Style::Fish | Style::Elvish, None) => {
                        writeln!(res, "{}", item.subst1)
                    }
                    (Style::Zsh | Style::Fish | Style::Elvish, Some(descr)) => {
                        writeln!(res, "{}\t{}", item.subst1, descr)
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
