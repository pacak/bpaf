#![allow(clippy::write_with_newline)]
#![allow(clippy::match_like_matches_macro)]
use std::collections::BTreeSet;

use crate::{
    info::Info,
    item::{Item, ShortLong},
    Meta,
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Metavar(pub(crate) &'static str);

impl std::fmt::Display for Metavar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char('<')?;
        f.write_str(self.0)?;
        f.write_char('>')
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum HelpItem<'a> {
    Decor {
        help: &'a str,
    },
    BlankDecor,
    Positional {
        strict: bool,
        metavar: Metavar,
        help: Option<&'a str>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<&'a str>,
        meta: &'a Meta,
        info: &'a Info,
    },
    Flag {
        name: ShortLongHelp,
        help: Option<&'a str>,
    },
    Argument {
        name: ShortLongHelp,
        metavar: Metavar,
        env: Option<&'static str>,
        help: Option<&'a str>,
    },
}

#[derive(Default, Debug)]
pub(crate) struct HelpItems<'a> {
    pub(crate) cmds: Vec<HelpItem<'a>>,
    pub(crate) psns: Vec<HelpItem<'a>>,
    pub(crate) flgs: Vec<HelpItem<'a>>,
}

impl HelpItem<'_> {
    fn is_decor(&self) -> bool {
        match self {
            HelpItem::Decor { .. } | HelpItem::BlankDecor => true,
            _ => false,
        }
    }
}

fn dedup(items: &mut BTreeSet<String>, payload: &mut String, prev: usize) {
    let new = payload[prev..payload.len()].to_owned();
    if !items.insert(new) {
        payload.truncate(prev);
    }
}

impl<'a> HelpItems<'a> {
    #[inline(never)]
    pub(crate) fn classify_item(&mut self, item: &'a Item) {
        match item {
            crate::item::Item::Positional {
                metavar,
                help,
                strict,
            } => {
                if help.is_some() {
                    self.psns.push(HelpItem::Positional {
                        metavar: Metavar(metavar),
                        help: help.as_deref(),
                        strict: *strict,
                    });
                }
            }
            crate::item::Item::Command {
                name,
                short,
                help,

                info,
                meta,
            } => {
                self.cmds.push(HelpItem::Command {
                    name,
                    info,
                    meta,
                    short: *short,
                    help: help.as_deref(),
                });
            }
            crate::item::Item::Flag {
                name,
                help,
                shorts: _,
            } => self.flgs.push(HelpItem::Flag {
                name: ShortLongHelp(*name),
                help: help.as_deref(),
            }),
            crate::item::Item::Argument {
                name,
                metavar,
                env,
                help,
                shorts: _,
            } => self.flgs.push(HelpItem::Argument {
                name: ShortLongHelp(*name),
                metavar: Metavar(metavar),
                env: *env,
                help: help.as_deref(),
            }),
        }
    }

    pub(crate) fn classify(&mut self, meta: &'a Meta) {
        match meta {
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    self.classify(x);
                }
            }
            Meta::HideUsage(x) | Meta::Optional(x) | Meta::Many(x) => self.classify(x),
            Meta::Item(item) => self.classify_item(item),

            Meta::Decorated(m, help) => {
                self.flgs.push(HelpItem::Decor { help });
                self.cmds.push(HelpItem::Decor { help });
                self.psns.push(HelpItem::Decor { help });
                self.classify(m);

                if self.flgs.last().map_or(false, HelpItem::is_decor) {
                    self.flgs.pop();
                } else {
                    self.flgs.push(HelpItem::BlankDecor);
                }

                if self.cmds.last().map_or(false, HelpItem::is_decor) {
                    self.cmds.pop();
                } else {
                    self.cmds.push(HelpItem::BlankDecor);
                }

                if self.psns.last().map_or(false, HelpItem::is_decor) {
                    self.psns.pop();
                } else {
                    self.psns.push(HelpItem::BlankDecor);
                }
            }
            Meta::Skip => (),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ShortLongHelp(pub(crate) ShortLong);

impl ShortLongHelp {
    #[inline]
    fn full_width(&self) -> usize {
        match self.0 {
            ShortLong::Short(_) => 2,
            ShortLong::Long(l) | ShortLong::ShortLong(_, l) => 6 + l.len(),
        }
    }
}

pub(crate) struct Long<'a>(pub(crate) &'a str);
impl std::fmt::Display for Long<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("--")?;
        f.write_str(self.0)
    }
}

pub(crate) struct Short(pub(crate) char);
impl std::fmt::Display for Short {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char('-')?;
        f.write_char(self.0)
    }
}

impl std::fmt::Display for ShortLongHelp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ShortLong::Short(short) => write!(f, "{}", w_flag!(Short(short))),
            ShortLong::Long(long) => write!(f, "    {}", w_flag!(Long(long))),
            ShortLong::ShortLong(short, long) => {
                write!(f, "{}, {}", w_flag!(Short(short)), w_flag!(Long(long)))
            }
        }
    }
}

/// supports padding of the help by some max width
impl std::fmt::Display for HelpItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpItem::Flag { name, help: _ } => write!(f, "    {}", name),
            HelpItem::Argument {
                name,
                metavar,
                help,
                env,
            } => {
                write!(f, "    {} {}", name, w_flag!(metavar))?;

                let width = f.width().unwrap();
                if let Some(env) = env {
                    let pad = width - self.full_width();

                    let m_os = std::env::var_os(env);
                    let val = match &m_os {
                        Some(s) => std::borrow::Cow::from(format!(" = {:?}", s.to_string_lossy())),
                        None => std::borrow::Cow::Borrowed(": N/A"),
                    };
                    let next_pad = 4 + self.full_width();
                    write!(f, "{:pad$}  [env:{}{}]", "", env, val, pad = pad,)?;
                    if help.is_some() {
                        write!(f, "\n{:width$}", "", width = next_pad)?;
                    }
                }
                Ok(())
            }
            HelpItem::Decor { help } => write!(f, "  {}", help),
            HelpItem::BlankDecor => Ok(()),
            HelpItem::Positional {
                metavar,
                help: _,
                strict,
            } => {
                if *strict {
                    write!(f, "    -- {}", w_flag!(metavar))
                } else {
                    write!(f, "    {}", w_flag!(metavar))
                }
            }
            HelpItem::Command {
                name,
                help: _,
                short,
                meta: _,
                info: _,
            } => match short {
                Some(s) => write!(f, "    {}, {}", w_flag!(name), w_flag!(s)),
                None => write!(f, "    {}", w_flag!(name)),
            },
        }?;

        // alt view requires width, so unwrap should just work;
        let width = f.width().unwrap();
        if let Some(help) = self.help() {
            let pad = width - self.full_width();
            for (ix, line) in help.split('\n').enumerate() {
                {
                    if ix == 0 {
                        write!(f, "{:pad$}  {}", "", line, pad = pad)
                    } else {
                        write!(f, "\n{:pad$}      {}", "", line, pad = width)
                    }
                }?;
            }
        }
        Ok(())
    }
}

impl<'a> From<&'a crate::item::Item> for HelpItem<'a> {
    fn from(item: &'a crate::item::Item) -> Self {
        match item {
            crate::item::Item::Positional {
                metavar,
                help,
                strict,
            } => Self::Positional {
                metavar: Metavar(metavar),
                strict: *strict,
                help: help.as_deref(),
            },
            crate::item::Item::Command {
                name,
                short,
                help,
                meta,
                info,
            } => Self::Command {
                name,
                short: *short,
                help: help.as_deref(),
                meta,
                info,
            },
            crate::item::Item::Flag {
                name,
                help,
                shorts: _,
            } => Self::Flag {
                name: ShortLongHelp(*name),
                help: help.as_deref(),
            },
            crate::item::Item::Argument {
                name,
                metavar,
                env,
                help,
                shorts: _,
            } => Self::Argument {
                name: ShortLongHelp(*name),
                metavar: Metavar(metavar),
                env: *env,
                help: help.as_deref(),
            },
        }
    }
}

impl HelpItem<'_> {
    #[must_use]
    /// Full width for the name, including implicit short flag, space and comma
    /// betwen short and log parameters and metavar variable if present
    fn full_width(&self) -> usize {
        match self {
            HelpItem::Decor { .. } | HelpItem::BlankDecor { .. } => 0,
            HelpItem::Flag { name, .. } => name.full_width(),
            HelpItem::Argument { name, metavar, .. } => name.full_width() + metavar.0.len() + 3,
            HelpItem::Positional { metavar, .. } => metavar.0.len() + 2,
            HelpItem::Command {
                name, short: None, ..
            } => name.len(),
            HelpItem::Command {
                name,
                short: Some(_),
                ..
            } => name.len() + 3,
        }
    }

    fn help(&self) -> Option<&str> {
        match self {
            HelpItem::Decor { .. } | HelpItem::BlankDecor => None,
            HelpItem::Command { help, .. }
            | HelpItem::Flag { help, .. }
            | HelpItem::Argument { help, .. }
            | HelpItem::Positional { help, .. } => *help,
        }
    }
}

pub(crate) fn render_help(
    info: &Info,
    parser_meta: &Meta,
    help_meta: &Meta,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;

    let mut res = String::new();
    if let Some(t) = info.descr {
        write!(res, "{}\n\n", t)?;
    }

    let auto = parser_meta.as_usage_meta().map(|u| u.to_string());
    if let Some(custom_usage) = info.usage {
        match auto {
            Some(auto_usage) => write!(
                res,
                "{}\n",
                custom_usage.replacen("{usage}", &auto_usage, 1)
            ),
            None => write!(res, "{}\n", custom_usage),
        }?;
    } else if let Some(usage) = auto {
        write!(res, "Usage: {}\n", usage)?;
    }

    if let Some(t) = info.header {
        write!(res, "\n{}\n", t)?;
    }

    let mut items = HelpItems::default();
    items.classify(parser_meta);
    items.classify(help_meta);
    let mut dedup_cache = BTreeSet::new();
    if !items.psns.is_empty() {
        let max_width = items
            .psns
            .iter()
            .map(HelpItem::full_width)
            .max()
            .unwrap_or(0);
        w_section!(res, "\nAvailable positional items:\n")?;
        for i in &items.psns {
            let len = res.len();
            write!(res, "{:padding$}\n", i, padding = max_width)?;
            dedup(&mut dedup_cache, &mut res, len);
        }
    }

    if !items.flgs.is_empty() {
        let max_width = items
            .flgs
            .iter()
            .map(HelpItem::full_width)
            .max()
            .unwrap_or(0);
        w_section!(res, "\nAvailable options:\n")?;
        for i in &items.flgs {
            let len = res.len();
            write!(res, "{:padding$}\n", i, padding = max_width)?;
            dedup(&mut dedup_cache, &mut res, len);
        }
    }

    if !items.cmds.is_empty() {
        w_section!(res, "\nAvailable commands:\n")?;
        let max_width = items
            .cmds
            .iter()
            .map(HelpItem::full_width)
            .max()
            .unwrap_or(0);
        for i in &items.cmds {
            let len = res.len();
            write!(res, "{:padding$}\n", i, padding = max_width)?;
            dedup(&mut dedup_cache, &mut res, len);
        }
    }
    if let Some(t) = info.footer {
        write!(res, "\n{}", t)?;
    }
    if !res.ends_with('\n') {
        res.push('\n');
    }
    Ok(res)
}
