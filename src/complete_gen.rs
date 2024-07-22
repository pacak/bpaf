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

// instead

// if we can't complete current value - don't make any suggestions at all!
// this behavior matches one from completions bash and zsh give for ls

use crate::{
    args::{Arg, State},
    complete_shell::{render_bash, render_fish, render_simple, render_test, render_zsh},
    item::ShortLong,
    meta_help::Metavar,
    parsers::NamedArg,
    Doc, ShellComp,
};
use std::ffi::OsStr;

#[derive(Clone, Debug)]
pub(crate) struct CurrentMeta {
    pub(crate) name: Metavar,
    pub(crate) help: Option<String>,
    /// Is metavar belongs to an argument?
    ///
    /// The difference is that for not arguments in a scenario like "[-v] <FILE>"
    /// While completing FILE It is also valid to suggest -v, while for arguments
    /// "-v -f <FILE>" and we are completing FILE suggesting -v won't be valid.
    ///
    /// As a result any metavar or any value with is_argument set disables any
    /// non argument values or metavars
    pub(crate) is_argument: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Complete {
    /// completions accumulated so far
    comps: Vec<Comp>,

    pub(crate) comps2: Vec<Comp>,

    /// Are we expanding a metavariable?
    /// This takes priority over comps
    pub(crate) meta: Option<CurrentMeta>,

    /// Output revision
    ///
    /// This value will be used to decide how to render the generated completion info
    /// for different shell.
    ///
    /// The only reason it is inside of Complete struct is that it gets created from arguments
    /// and needs to be stored somewhere
    pub(crate) output_rev: usize,

    /// Argument that is being completed
    ///
    /// This argument can be either positional or named item, or even part of them like "--ver"
    /// in a process of being complete to "--verbose"
    pub(crate) current_arg: String,

    /// Current argument was consumed by a positional parser
    pub(crate) consumed_as_positional: bool,
}

impl Complete {
    pub(crate) fn new(output_rev: usize, current_arg: String) -> Self {
        Self {
            comps: Vec::new(),
            meta: None,
            comps2: Vec::new(),
            output_rev,
            current_arg,
            consumed_as_positional: false,
        }
    }
}

impl State {
    /// Add a new completion hint for flag, if needed
    pub(crate) fn push_flag(&mut self, named: &NamedArg) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            if let Ok(name) = ShortLong::try_from(named) {
                comp.comps.push(Comp::Flag {
                    extra: CompExtra {
                        depth,
                        group: None,
                        help: named.help.as_ref().and_then(Doc::to_completion),
                    },
                    name,
                });
            }
        }
    }

    /// Add a new completion hint for an argument, if needed
    pub(crate) fn push_argument(&mut self, named: &NamedArg, metavar: &'static str) {
        let depth = self.depth();
        if let Some(comp) = self.comp_mut() {
            if let Ok(name) = ShortLong::try_from(named) {
                comp.comps.push(Comp::Argument {
                    extra: CompExtra {
                        depth,
                        group: None,
                        help: named.help.as_ref().and_then(Doc::to_completion),
                    },
                    metavar,
                    name,
                });
            }
        }
    }

    /// Add a new completion hint for metadata, if needed
    ///
    /// `is_argument` is set to true when we are trying to parse the value and false if
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
    pub(crate) fn push_with_group(&mut self, group: &Option<String>, comps: &mut Vec<Comp>) {
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

impl Complete {
    pub(crate) fn push_shell(&mut self, op: ShellComp, is_argument: bool, depth: usize) {
        self.comps.push(Comp::Shell {
            extra: CompExtra {
                depth,
                group: None,
                help: None,
            },
            script: op,
            is_argument,
        });
    }

    pub(crate) fn push_value(
        &mut self,
        body: String,
        help: Option<String>,
        group: Option<String>,
        depth: usize,
        is_argument: bool,
    ) {
        self.comps.push(Comp::Value {
            body,
            is_argument,
            extra: CompExtra { depth, group, help },
        });
    }

    pub(crate) fn push_comp(&mut self, comp: Comp) {
        self.comps.push(comp);
    }

    pub(crate) fn extend_comps(&mut self, comps: Vec<Comp>) {
        self.comps2.extend(comps);
    }

    pub(crate) fn drain_comps(&mut self) -> std::vec::Drain<Comp> {
        self.comps2.drain(0..)
    }

    pub(crate) fn swap_comps(&mut self, other: &mut Vec<Comp>) {
        std::mem::swap(other, &mut self.comps2);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompExtra {
    /// Used by complete_gen to separate commands from each other
    pub(crate) depth: usize,

    /// Render this option in a group along with all other items with the same name
    pub(crate) group: Option<String>,

    /// help message attached to a completion item
    pub(crate) help: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    /// short or long flag
    Flag { extra: CompExtra, name: ShortLong },

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
        /// AKA not positional
        is_argument: bool,
    },

    Shell {
        extra: CompExtra,
        script: ShellComp,
        /// AKA not positional
        is_argument: bool,
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
pub(crate) struct ShowComp<'a> {
    /// value to be actually inserted by the autocomplete system
    pub(crate) subst: String,

    /// pretty rendering which might include metavars, etc
    pub(crate) pretty: String,

    pub(crate) extra: &'a CompExtra,
}

impl std::fmt::Display for ShowComp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(help), true) = (&self.extra.help, self.subst.is_empty()) {
            write!(f, "{}: {}", self.pretty, help)
        } else if let Some(help) = &self.extra.help {
            write!(f, "{:24} -- {}", self.pretty, help)
        } else {
            write!(f, "{}", self.pretty)
        }
    }
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

/// What is the preceeding item, if any
///
/// Mostly is there to tell if we are trying to complete and argument or not...
#[derive(Debug, Copy, Clone)]
enum Prefix<'a> {
    NA,
    Short(char),
    Long(&'a str),
}

impl State {
    /// Generate completion from collected heads
    ///
    /// before calling this method we run parser in "complete" mode and collect live heads inside
    /// `self.comp`, this part goes over collected heads and generates possible completions from
    /// that
    pub(crate) fn check_complete(&self) -> Option<String> {
        let comp = self.comp_ref()?;

        /*
                let mut items = self
                    .items
                    .iter()
                    .rev()
                    .filter_map(Arg::and_os_string)
                    .filter_map(pair_to_os_string);
        */
        // try get a current item to complete - must be non-virtual right most one
        // value must be present here, and can fail only for non-utf8 values
        // can't do much completing with non-utf8 values since bpaf needs to print them to stdout
        //        let (cur, lit) = items.next()?;

        // For cases like "-k=val", "-kval", "--key=val", "--key val"
        // last value is going  to be either Arg::Word or Arg::ArgWord
        // so to perform full completion we look at the preceeding item
        // and use it's value if it was a composite short/long argument
        //        let preceeding = items.next();
        //        let (pos_only, full_lit) = match preceeding {
        //            Some((Arg::Short(_, true, _os) | Arg::Long(_, true, _os), full_lit)) => {
        //                (false, full_lit)
        //            }
        //            Some((Arg::PosWord(_), _)) => (true, lit),
        //            _ => (false, lit),
        //        };

        //        let is_named = match cur {
        //            Arg::Short(_, _, _) | Arg::Long(_, _, _) => true,
        //            Arg::ArgWord(_) | Arg::Word(_) | Arg::PosWord(_) => false,
        //        };

        //        let prefix = match preceeding {
        //            Some((Arg::Short(s, true, _os), _lit)) => Prefix::Short(*s),
        //            Some((Arg::Long(l, true, _os), _lit)) => Prefix::Long(l.as_str()),
        //            _ => Prefix::NA,
        //        };

        //        println!("comps2: {:?}", comp.comps2);

        let (items, shell) = comp.complete("", false, false, Prefix::NA);

        Some(match comp.output_rev {
            0 => render_test(&items, &shell ),
            1 => render_simple(&items), // <- AKA elvish
            7 => render_zsh(&items, &shell ),
            8 => render_bash(&items, &shell ),
            9 => render_fish(&items, &shell , self.path[0].as_str()),
            unk => {
                #[cfg(debug_assertions)]
                {
                    eprintln!("Unsupported output revision {}, you need to genenerate your shell completion files for the app", unk);
                    std::process::exit(1);
                }
                #[cfg(not(debug_assertions))]
                {
                    std::process::exit(0);
                }
            }
        }.unwrap())
    }
}

/// Try to expand short string names into long names if possible
fn preferred_name(name: ShortLong) -> String {
    match name {
        ShortLong::Short(s) => format!("-{}", s),
        ShortLong::Long(l) | ShortLong::Both(_, l) => format!("--{}", l),
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
        ShortLong::Short(s) | ShortLong::Both(s, _) => {
            can_match |= arg
                .strip_prefix('-')
                .and_then(|a| a.strip_prefix(s))
                .map_or(false, str::is_empty);
        }
    }

    // and long string too
    match name {
        ShortLong::Short(_) => {}
        ShortLong::Long(l) | ShortLong::Both(_, l) => {
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
            Comp::Metavariable { is_argument, .. }
            | Comp::Value { is_argument, .. }
            | Comp::Shell { is_argument, .. } => *is_argument,
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

impl State {
    /// Move current metavariable contents into completions
    ///
    /// This requires &mut access to state
    pub(crate) fn convert_current_metavar(&mut self) {
        if let Some(comp) = self.comp_mut() {
            if let Some(meta) = std::mem::take(&mut comp.meta) {
                if meta.is_argument {
                    comp.comps2.clear();
                }
                comp.comps2.push(Comp::Metavariable {
                    extra: CompExtra {
                        depth: 0,
                        group: None,
                        help: meta.help.clone(),
                    },
                    meta: meta.name.0,
                    is_argument: meta.is_argument,
                });
            }
        }
    }
}

impl Complete {
    fn complete(
        &self,
        arg: &str,
        pos_only: bool,
        is_named: bool,
        prefix: Prefix,
    ) -> (Vec<ShowComp>, Vec<ShellComp>) {
        let mut items: Vec<ShowComp> = Vec::new();
        let mut shell = Vec::new();
        //        let max_depth = self.comps.iter().map(Comp::depth).max().unwrap_or(0);
        let mut only_values = self.comps2.iter().any(|v| {
            matches!(
                v,
                Comp::Value {
                    is_argument: true,
                    ..
                } | Comp::Metavariable {
                    is_argument: true,
                    ..
                }
            )
        });

        for item in self.comps2.iter()
        //            .filter(|c| c.depth() == max_depth && (!pos_only || c.is_pos()))
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
                        subst: match prefix {
                            Prefix::NA => body.clone(),
                            Prefix::Short(s) => format!("-{}={}", s, body),
                            Prefix::Long(l) => format!("--{}={}", l, body),
                        },
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
                        subst: String::new(),
                        pretty: (*meta).to_string(),
                        extra,
                    });
                }

                Comp::Shell { script, .. } => {
                    if !is_named {
                        shell.push(*script);
                    }
                }
            }
        }

        (items, shell)
    }
}
