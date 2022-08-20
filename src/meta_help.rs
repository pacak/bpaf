#![allow(clippy::write_with_newline)]
use crate::{info::Info, item::ShortLong, Meta};

#[derive(Debug)]
enum HelpItem<'a> {
    Decor {
        help: &'a str,
    },
    BlankDecor,
    Positional {
        metavar: &'static str,
        help: Option<&'a str>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<&'a str>,
    },
    Flag {
        name: ShortLong,
        help: Option<&'a str>,
    },
    Argument {
        name: ShortLong,
        metavar: &'static str,
        env: Option<&'static str>,
        help: Option<&'a str>,
    },
}

/// supports padding of the help by some max width
impl std::fmt::Display for HelpItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelpItem::Flag { name, help: _ } => write!(f, "    {:#}", name),
            HelpItem::Argument {
                name,
                metavar,
                help: _,
                env,
            } => {
                write!(f, "    {:#} <{}>", name, metavar)?;

                let width = f.width().unwrap();
                if let Some(env) = env {
                    let pad = width - self.full_width();
                    let val = match std::env::var(env) {
                        Ok(val) => format!(" = {:?}", val),
                        Err(std::env::VarError::NotPresent) => ": N/A".to_string(),
                        Err(std::env::VarError::NotUnicode(_)) => {
                            ": current value is not utf8".to_string()
                        }
                    };
                    let next_pad = 4 + self.full_width();
                    write!(
                        f,
                        "{:pad$}  [env:{}{}]\n{:width$}",
                        "",
                        env,
                        val,
                        "",
                        pad = pad,
                        width = next_pad,
                    )?;
                }
                Ok(())
            }
            HelpItem::Decor { .. } => write!(f, "    "),
            HelpItem::BlankDecor => Ok(()),
            HelpItem::Positional { metavar, help: _ } => write!(f, "    <{}>", metavar),
            HelpItem::Command {
                name,
                help: _,
                short,
            } => match short {
                Some(s) => write!(f, "    {}, {}", name, s),
                None => write!(f, "    {}", name),
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
            crate::item::Item::Positional { metavar, help } => Self::Positional {
                metavar,
                help: help.as_deref(),
            },
            crate::item::Item::Command { name, short, help } => Self::Command {
                name,
                short: *short,
                help: help.as_deref(),
            },
            crate::item::Item::Flag { name, help } => Self::Flag {
                name: *name,
                help: help.as_deref(),
            },
            crate::item::Item::Argument {
                name,
                metavar,
                env,
                help,
            } => Self::Argument {
                name: *name,
                metavar,
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
            HelpItem::Decor { .. } => 0,
            HelpItem::BlankDecor { .. } => 0,
            HelpItem::Flag { name, .. } => name.full_width(),
            HelpItem::Argument { name, metavar, .. } => name.full_width() + metavar.len() + 3,
            HelpItem::Positional { metavar, .. } => metavar.len() + 2,
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
            HelpItem::Decor { help } => Some(help),
            HelpItem::BlankDecor => None,
            HelpItem::Command { help, .. }
            | HelpItem::Flag { help, .. }
            | HelpItem::Argument { help, .. }
            | HelpItem::Positional { help, .. } => *help,
        }
    }
}

impl Meta {
    fn commands(&self) -> Vec<HelpItem> {
        let mut res = Vec::new();
        self.collect_items(&mut res, crate::item::Item::is_command);
        res
    }

    fn flags(&self) -> Vec<HelpItem> {
        let mut res = Vec::new();
        self.collect_items(&mut res, crate::item::Item::is_flag);
        res
    }

    fn poss(&self) -> Vec<HelpItem> {
        let mut res = Vec::new();
        self.collect_items(&mut res, crate::item::Item::is_positional);
        res
    }

    fn collect_items<'a, F>(&'a self, res: &mut Vec<HelpItem<'a>>, pred: F)
    where
        F: Fn(&crate::item::Item) -> bool + Copy,
    {
        match self {
            Meta::Skip => {}
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    x.collect_items(res, pred);
                }
            }
            Meta::Many(a) | Meta::Optional(a) => a.collect_items(res, pred),
            Meta::Item(i) => {
                if pred(i) {
                    res.push(HelpItem::from(i));
                }
            }
            Meta::Decorated(x, msg) => {
                res.push(HelpItem::Decor { help: msg.as_ref() });
                let prev_len = res.len();
                x.collect_items(res, pred);
                if res.len() == prev_len {
                    res.pop();
                } else {
                    res.push(HelpItem::BlankDecor);
                }
            }
        }
    }
}

pub(crate) fn render_help(
    info: &Info,
    parser_meta: Meta,
    help_meta: Meta,
) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;

    let mut res = String::new();
    if let Some(t) = info.descr {
        write!(res, "{}\n\n", t)?;
    }
    if let Some(u) = info.usage {
        write!(res, "{}\n", u)?;
    } else if let Some(usage) = parser_meta.as_usage_meta() {
        write!(res, "Usage: {}\n", usage)?;
    }
    if let Some(t) = info.header {
        write!(res, "\n{}\n", t)?;
    }
    let meta = Meta::And(vec![parser_meta, help_meta]);

    let poss = &meta.poss();
    if !poss.is_empty() {
        let max_width = poss.iter().map(HelpItem::full_width).max().unwrap_or(0);
        write!(res, "\nAvailable positional items:\n")?;
        for i in poss {
            write!(res, "{:padding$}\n", i, padding = max_width)?;
        }
    }

    let flags = &meta.flags();
    if !flags.is_empty() {
        let max_width = flags.iter().map(HelpItem::full_width).max().unwrap_or(0);
        write!(res, "\nAvailable options:\n")?;
        for i in flags {
            write!(res, "{:padding$}\n", i, padding = max_width)?;
        }
    }

    let commands = &meta.commands();
    if !commands.is_empty() {
        write!(res, "\nAvailable commands:\n")?;
        let max_width = commands.iter().map(HelpItem::full_width).max().unwrap_or(0);
        for i in commands {
            write!(res, "{:padding$}\n", i, padding = max_width)?;
        }
    }
    if let Some(t) = info.footer {
        write!(res, "\n{}", t)?;
    }
    Ok(res)
}
