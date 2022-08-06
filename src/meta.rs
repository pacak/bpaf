use crate::{item::Item, item::ShortLong};
use std::fmt::Write;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Meta {
    And(Vec<Meta>),
    Or(Vec<Meta>),
    Optional(Box<Meta>),
    Item(Item),
    Many(Box<Meta>),
    Decorated(Box<Meta>, String),
    Skip,
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_usage_meta() {
            Some(usage) => usage.fmt(f),
            None => f.write_str("no parameters expected"),
        }
    }
}

impl Meta {
    pub(crate) fn optional(self) -> Self {
        Self::Optional(Box::new(self))
    }

    pub(crate) fn many(self) -> Self {
        Self::Many(Box::new(self))
    }

    pub(crate) fn commands(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res, Item::is_command);
        res
    }

    pub(crate) fn flags(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res, Item::is_flag);
        res
    }

    fn alts(self, to: &mut Vec<Meta>) {
        match self {
            Meta::Or(mut xs) => to.append(&mut xs),
            Meta::Skip => {}
            meta => to.push(meta),
        }
    }

    pub(crate) fn or(self, other: Meta) -> Self {
        let mut res = Vec::new();
        self.alts(&mut res);
        other.alts(&mut res);
        match res.len() {
            0 => Meta::Skip,
            1 => res.remove(0),
            _ => Meta::Or(res),
        }
    }

    #[must_use]
    pub(crate) fn decorate<M>(self, msg: M) -> Self
    where
        M: Into<String>,
    {
        Meta::Decorated(Box::new(self), msg.into())
    }

    fn collect_items<F>(&self, res: &mut Vec<Item>, pred: F)
    where
        F: Fn(&Item) -> bool + Copy,
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
                    res.push(i.clone());
                }
            }
            Meta::Decorated(x, msg) => {
                res.push(Item::decoration(Some(msg)));
                let prev_len = res.len();
                x.collect_items(res, pred);
                if res.len() == prev_len {
                    res.pop();
                } else {
                    res.push(Item::decoration(None::<String>));
                }
            }
        }
    }

    /// Represent [`Meta`] as [`UsageMeta`]
    ///
    /// `None` indicates that command takes no parameters so usage line is not shown
    pub(crate) fn as_usage_meta(&self) -> Option<UsageMeta> {
        let mut had_commands = false;
        collect_usage_meta(self, true, &mut had_commands)
    }
}

#[derive(Debug)]
pub(crate) enum UsageMeta {
    And(Vec<Self>),
    Or(Vec<Self>),
    Required(Box<Self>),
    Optional(Box<Self>),
    Many(Box<Self>),
    Short(char, Option<&'static str>),
    Long(&'static str, Option<&'static str>),
    Pos(&'static str),
    Command,
}

/// Returns number of collected elements and if they are required
///
/// parameter `required` defines the context value is used in: optional or required. Optional
/// values in required context will be surrounded by []
///
/// parameter `had_commands` is used for command deduplication from or groups, should be initialized with false
fn collect_usage_meta(meta: &Meta, required: bool, had_commands: &mut bool) -> Option<UsageMeta> {
    Some(match meta {
        Meta::And(xs) => {
            // even if whole group is optional - all the items inside are required
            // to construct it
            let mut items = xs
                .iter()
                .filter_map(|x| collect_usage_meta(x, true, had_commands))
                .collect::<Vec<_>>();
            match items.len() {
                0 => return None,
                1 => UsageMeta::maybe_optional(items.remove(0), required),
                _ => UsageMeta::maybe_optional(UsageMeta::And(items), required),
            }
        }

        Meta::Or(xs) => {
            let mut had_commands = false;
            // if the whole group is optional - any item inside is in optional context:
            // no need to show [] if they are present.
            let mut items = xs
                .iter()
                .filter_map(|x| collect_usage_meta(x, required, &mut had_commands))
                .collect::<Vec<_>>();

            match items.len() {
                0 => return None,
                1 => items.remove(0),
                _ => UsageMeta::maybe_required(UsageMeta::Or(items), required),
            }
        }
        Meta::Optional(m) => {
            let inner = collect_usage_meta(m, false, had_commands)?;
            if required {
                UsageMeta::Optional(Box::new(inner))
            } else {
                inner
            }
        }
        Meta::Many(meta) => {
            UsageMeta::Many(Box::new(collect_usage_meta(meta, required, had_commands)?))
        }
        Meta::Decorated(meta, _) => collect_usage_meta(meta, required, had_commands)?,
        Meta::Skip => return None,
        Meta::Item(i) => match i {
            Item::Decor { help: _ } => return None,
            Item::Positional { metavar } => UsageMeta::Pos(metavar),
            Item::Command { name: _, help: _ } => {
                if *had_commands {
                    return None;
                }
                *had_commands = true;
                UsageMeta::Command
            }
            Item::Flag { name, help: _ } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => UsageMeta::Short(*s, None),
                ShortLong::Long(l) => UsageMeta::Long(l, None),
            },
            Item::Argument {
                name,
                metavar,
                env: _,
                help: _,
            } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => {
                    UsageMeta::Short(*s, Some(metavar))
                }
                ShortLong::Long(l) => UsageMeta::Long(l, Some(metavar)),
            },
        },
    })
}

impl UsageMeta {
    fn maybe_required(self, required: bool) -> Self {
        if required {
            Self::Required(Box::new(self))
        } else {
            self
        }
    }

    fn maybe_optional(self, required: bool) -> Self {
        if required {
            self
        } else {
            Self::Optional(Box::new(self))
        }
    }
}

impl std::fmt::Display for UsageMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageMeta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        f.write_char(' ')?;
                    }
                    x.fmt(f)?;
                }
                Ok(())
            }
            UsageMeta::Or(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        f.write_str(" | ")?;
                    }
                    x.fmt(f)?;
                }
                Ok(())
            }
            UsageMeta::Required(x) => write!(f, "({x})"),
            UsageMeta::Optional(x) => write!(f, "[{x}]"),
            UsageMeta::Many(x) => write!(f, "{x}..."),
            UsageMeta::Short(c, None) => write!(f, "-{c}"),
            UsageMeta::Long(l, None) => write!(f, "--{l}"),
            UsageMeta::Short(c, Some(v)) => write!(f, "-{c} {v}"),
            UsageMeta::Long(l, Some(v)) => write!(f, "--{l} {v}"),
            UsageMeta::Command => f.write_str("COMMAND ..."),
            UsageMeta::Pos(s) => write!(f, "<{s}>"),
        }
    }
}
