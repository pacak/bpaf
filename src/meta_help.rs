#![allow(clippy::write_with_newline)]
#![allow(clippy::match_like_matches_macro)]
use std::collections::BTreeSet;

use crate::{
    buffer::{Buffer, Checkpoint, Style},
    info::Info,
    item::{Item, ShortLong},
    meta::DecorPlace,
    Meta,
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct Metavar(pub(crate) &'static str);

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
        margin: DecorPlace,
    },
    BlankDecor,
    Positional {
        metavar: Metavar,
        help: Option<&'a str>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<&'a str>,
        #[cfg(feature = "manpage")]
        meta: &'a Meta,
        #[cfg(feature = "manpage")]
        info: &'a Info,
    },
    Flag {
        name: ShortLong,
        env: Option<&'static str>,
        help: Option<&'a str>,
    },
    Argument {
        name: ShortLong,
        metavar: Metavar,
        env: Option<&'static str>,
        help: Option<&'a str>,
    },
    MultiArg {
        name: ShortLong,
        help: Option<&'a str>,
        fields: &'a [(Metavar, Option<String>)],
    },
}

#[derive(Default, Debug)]
/// A collection of all the help items separated into flags, positionals and commands
///
/// Items are stored as references and can be trivially copied
pub(crate) struct HelpItems<'a> {
    pub(crate) flgs: Vec<HelpItem<'a>>,
    pub(crate) psns: Vec<HelpItem<'a>>,
    pub(crate) cmds: Vec<HelpItem<'a>>,
}

impl HelpItem<'_> {
    fn is_decor(&self) -> bool {
        match self {
            HelpItem::Decor { .. } | HelpItem::BlankDecor => true,
            _ => false,
        }
    }
}

fn dedup(items: &mut BTreeSet<String>, buf: &mut Buffer, cp: Checkpoint) {
    let new = buf.content_since(cp).to_owned();
    if !items.insert(new) {
        buf.rollback(cp);
    }
}

impl<'a> HelpItems<'a> {
    #[inline(never)]
    /// Store a reference to this item into corresponding class - flag, positional or command
    pub(crate) fn classify_item(&mut self, item: &'a Item) {
        match item {
            Item::Positional {
                metavar,
                help,
                strict: _,
            } => {
                if help.is_some() {
                    self.psns.push(HelpItem::Positional {
                        metavar: *metavar,
                        help: help.as_deref(),
                    });
                }
            }
            Item::Command {
                name,
                short,
                help,

                #[cfg(not(feature = "manpage"))]
                    meta: _,
                #[cfg(not(feature = "manpage"))]
                    info: _,
                #[cfg(feature = "manpage")]
                info,
                #[cfg(feature = "manpage")]
                meta,
            } => {
                self.cmds.push(HelpItem::Command {
                    name,
                    #[cfg(feature = "manpage")]
                    info,
                    #[cfg(feature = "manpage")]
                    meta,
                    short: *short,
                    help: help.as_deref(),
                });
            }
            Item::Flag {
                name,
                help,
                env,
                shorts: _,
            } => self.flgs.push(HelpItem::Flag {
                name: *name,
                env: *env,
                help: help.as_deref(),
            }),
            Item::Argument {
                name,
                metavar,
                env,
                help,
                shorts: _,
            } => self.flgs.push(HelpItem::Argument {
                name: *name,
                metavar: *metavar,
                env: *env,
                help: help.as_deref(),
            }),
            Item::MultiArg {
                name,
                shorts: _,
                help,
                fields,
            } => self.flgs.push(HelpItem::MultiArg {
                name: *name,
                help: help.as_deref(),
                fields,
            }),
        }
    }

    /// Recursively classify contents of the Meta
    pub(crate) fn classify(&mut self, meta: &'a Meta) {
        match meta {
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    self.classify(x);
                }
            }
            Meta::HideUsage(x) | Meta::Optional(x) | Meta::Many(x) => self.classify(x),
            Meta::Item(item) => self.classify_item(item),

            Meta::Decorated(m, help, DecorPlace::Header) => {
                self.flgs.push(HelpItem::Decor {
                    help,
                    margin: DecorPlace::Header,
                });
                self.cmds.push(HelpItem::Decor {
                    help,
                    margin: DecorPlace::Header,
                });
                self.psns.push(HelpItem::Decor {
                    help,
                    margin: DecorPlace::Header,
                });
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
            Meta::Decorated(m, help, DecorPlace::Suffix) => {
                let flgs = self.flgs.len();
                let cmds = self.cmds.len();
                let psns = self.psns.len();
                self.classify(m);
                let xs = if flgs != self.flgs.len() {
                    &mut self.flgs
                } else if psns != self.psns.len() {
                    &mut self.psns
                } else if cmds != self.cmds.len() {
                    &mut self.cmds
                } else {
                    return;
                };
                xs.push(HelpItem::Decor {
                    help,
                    margin: DecorPlace::Suffix,
                });
            }
            Meta::Skip => (),
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

impl<'a> From<&'a crate::item::Item> for HelpItem<'a> {
    fn from(item: &'a crate::item::Item) -> Self {
        match item {
            Item::Positional {
                metavar,
                help,
                strict: _,
            } => Self::Positional {
                metavar: *metavar,
                help: help.as_deref(),
            },
            Item::Command {
                name,
                short,
                help,
                #[cfg(not(feature = "manpage"))]
                    meta: _,
                #[cfg(not(feature = "manpage"))]
                    info: _,
                #[cfg(feature = "manpage")]
                meta,
                #[cfg(feature = "manpage")]
                info,
            } => Self::Command {
                name,
                short: *short,
                help: help.as_deref(),
                #[cfg(feature = "manpage")]
                meta,
                #[cfg(feature = "manpage")]
                info,
            },
            Item::Flag {
                name,
                env,
                help,
                shorts: _,
            } => Self::Flag {
                name: *name,
                env: *env,
                help: help.as_deref(),
            },
            Item::Argument {
                name,
                metavar,
                env,
                help,
                shorts: _,
            } => Self::Argument {
                name: *name,
                metavar: *metavar,
                env: *env,
                help: help.as_deref(),
            },
            Item::MultiArg {
                name,
                shorts: _,
                help,
                fields,
            } => Self::MultiArg {
                name: *name,
                help: help.as_deref(),
                fields,
            },
        }
    }
}

fn write_metavar(buf: &mut Buffer, metavar: Metavar) {
    buf.write_char('<', Style::Label);
    buf.write_str(metavar.0, Style::Label);
    buf.write_char('>', Style::Label);
}

fn write_help_item(buf: &mut Buffer, item: &HelpItem) {
    match item {
        HelpItem::Decor { help, margin } => {
            match margin {
                DecorPlace::Header => {
                    buf.margin(2);
                }
                DecorPlace::Suffix => {
                    buf.tabstop();
                }
            }
            buf.write_str(help, Style::Text);
        }
        HelpItem::BlankDecor => {}
        HelpItem::Positional { metavar, help } => {
            buf.margin(4);
            write_metavar(buf, *metavar);
            if let Some(help) = help {
                buf.tabstop();
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::Command {
            name,
            short,
            help,
            #[cfg(feature = "manpage")]
                meta: _,
            #[cfg(feature = "manpage")]
                info: _,
        } => {
            buf.margin(4);
            buf.write_str(name, Style::Label);
            if let Some(short) = short {
                buf.write_str(", ", Style::Text);
                buf.write_char(*short, Style::Label);
            }
            buf.tabstop();
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::Flag { name, env, help } => {
            buf.margin(4);
            write_shortlong(buf, *name);
            buf.tabstop();
            if let Some(env) = env {
                let val = if std::env::var_os(env).is_some() {
                    ": set"
                } else {
                    ": not set"
                };
                buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
                if help.is_some() {
                    buf.newline();
                    buf.tabstop();
                }
            }
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::Argument {
            name,
            metavar,
            env,
            help,
        } => {
            buf.margin(4);
            write_shortlong(buf, *name);
            buf.write_str(" ", Style::Label);
            write_metavar(buf, *metavar);
            buf.tabstop();

            if let Some(env) = env {
                let val = match std::env::var_os(env) {
                    Some(s) => std::borrow::Cow::from(format!(" = {:?}", s.to_string_lossy())),
                    None => std::borrow::Cow::Borrowed(": N/A"),
                };
                buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
                if help.is_some() {
                    buf.newline();
                    buf.tabstop();
                }
            }
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::MultiArg { name, help, fields } => {
            buf.margin(4);
            write_shortlong(buf, *name);
            for (field, _) in fields.iter() {
                buf.write_str(" ", Style::Label);
                write_metavar(buf, *field);
            }

            if let Some(help) = help {
                buf.tabstop();
                buf.write_str(help, Style::Text);
            }
            buf.margin(12);
            for (field, help) in fields.iter() {
                if let Some(help) = help {
                    buf.newline();
                    write_metavar(buf, *field);
                    buf.tabstop();
                    buf.write_str(help, Style::Text);
                }
            }
        }
    }
    buf.newline();
}

fn write_shortlong(buf: &mut Buffer, name: ShortLong) {
    match name {
        ShortLong::Short(s) => {
            buf.write_char('-', Style::Label);
            buf.write_char(s, Style::Label);
        }
        ShortLong::Long(l) => {
            buf.write_str("    --", Style::Label);
            buf.write_str(l, Style::Label);
        }
        ShortLong::ShortLong(s, l) => {
            buf.write_char('-', Style::Label);
            buf.write_char(s, Style::Label);
            buf.write_str(", ", Style::Text);
            buf.write_str("--", Style::Label);
            buf.write_str(l, Style::Label);
        }
    }
}

fn write_as_lines(buf: &mut Buffer, line: &str) {
    for line in line.lines() {
        buf.write_str(line, Style::Text);
        buf.newline();
    }
}

fn write_items(items: &[HelpItem], descr: &str) -> String {
    if items.is_empty() {
        String::new()
    } else {
        let mut buf = Buffer::default();
        let mut dedup_cache: BTreeSet<String> = BTreeSet::new();
        buf.newline();
        buf.margin(0);
        buf.write_str(descr, Style::Section);
        buf.newline();
        for i in items {
            let cp = buf.checkpoint();
            write_help_item(&mut buf, i);
            dedup(&mut dedup_cache, &mut buf, cp);
        }
        buf.to_string()
    }
}

pub fn render_help(info: &Info, parser_meta: &Meta, help_meta: &Meta) -> String {
    let mut res = String::new();
    let mut buf = Buffer::default();

    if let Some(t) = info.descr {
        write_as_lines(&mut buf, t);
        buf.newline();
    }

    let auto = parser_meta.to_usage_meta().map(|u| u.to_string());
    if let Some(custom_usage) = info.usage {
        match auto {
            Some(auto_usage) => buf.write_str(
                custom_usage.replacen("{usage}", &auto_usage, 1).as_str(),
                Style::Text,
            ),
            None => buf.write_str(custom_usage, Style::Text),
        };
        buf.newline();
    } else if let Some(usage) = auto {
        buf.write_str("Usage: ", Style::Text);
        buf.write_str(&usage, Style::Text);
        buf.newline();
    }

    if let Some(t) = info.header {
        buf.newline();
        buf.margin(0);

        write_as_lines(&mut buf, t);
    }

    let mut items = HelpItems::default();
    items.classify(parser_meta);
    items.classify(help_meta);

    res.push_str(&buf.to_string());

    res.push_str(&write_items(&items.psns, "Available positional items:"));
    res.push_str(&write_items(&items.flgs, "Available options:"));
    res.push_str(&write_items(&items.cmds, "Available commands:"));

    let mut buf = Buffer::default();
    if let Some(footer) = info.footer {
        buf.margin(0);
        buf.newline();
        write_as_lines(&mut buf, footer);
    }

    res.push_str(&buf.to_string());
    res
}
