use crate::{
    item::{Item, ShortLong},
    Meta,
};

#[derive(Debug)]
pub(crate) enum UsageMeta {
    And(Vec<Self>),
    Or(Vec<Self>),
    Required(Box<Self>),
    Optional(Box<Self>),
    Many(Box<Self>),
    ShortFlag(char),
    ShortArg(char, &'static str),
    LongFlag(&'static str),
    LongArg(&'static str, &'static str),
    Pos(&'static str),
    StrictPos(&'static str),
    Command,
}

// positional validity rules:
// - any meta containing a positional becomes positional itself
// - positional item must occupy the right most position inside Meta::And

pub(crate) fn to_usage_meta(meta: &Meta) -> Option<UsageMeta> {
    let mut had_commands = false;
    let mut is_pos = false;

    collect_usage_meta(meta, true, &mut had_commands, &mut is_pos)
}
/// Transforms `Meta` to [`UsageMeta`]
///
/// parameter `required` defines the value's context: optional or required.
/// `bpaf` shows Optional values in required context in []
///
/// `bpaf` uses parameter `had_commands` for command deduplication, initialize it with false
/// `bpaf` uses parameter `is_pos` for positional validation, initialize it with false
///
/// return value is None if parser takes no parameters at all
fn collect_usage_meta(
    meta: &Meta,
    required: bool,
    had_commands: &mut bool,
    is_pos: &mut bool,
) -> Option<UsageMeta> {
    Some(match meta {
        Meta::And(xs) => {
            // even if whole group is optional - it needs all the items inside to construct it
            let mut items = xs
                .iter()
                .filter_map(|x| {
                    let mut this_pos = false;
                    let usage_meta = collect_usage_meta(x, true, had_commands, &mut this_pos)?;
                    assert!(!*is_pos || this_pos,
                        "bpaf usage BUG: all positional and command items must be placed in the right \
                        most position of the structure or tuple they are in but {} breaks this rule. \
                        See bpaf documentation for `positional` and `positional_os` for details.",
                        usage_meta
                    );

                    *is_pos |= this_pos;
                    Some(usage_meta)
                })
                .collect::<Vec<_>>();
            match items.len() {
                0 => return None,
                1 => UsageMeta::maybe_optional(items.remove(0), required),
                _ => UsageMeta::maybe_optional(UsageMeta::And(items), required),
            }
        }

        Meta::Or(xs) => {
            // if the whole group is optional - any item inside is in optional context:
            // no need to show [] if they're present.
            let mut items = xs
                .iter()
                .filter_map(|x| collect_usage_meta(x, required, had_commands, is_pos))
                .collect::<Vec<_>>();

            match items.len() {
                0 => return None,
                1 => items.remove(0),
                _ => UsageMeta::maybe_required(UsageMeta::Or(items), required),
            }
        }
        Meta::Optional(m) => {
            let inner = collect_usage_meta(m, false, had_commands, is_pos)?;
            if required {
                UsageMeta::Optional(Box::new(inner))
            } else {
                inner
            }
        }
        Meta::Many(meta) => UsageMeta::Many(Box::new(collect_usage_meta(
            meta,
            required,
            had_commands,
            is_pos,
        )?)),
        Meta::Decorated(meta, _) => collect_usage_meta(meta, required, had_commands, is_pos)?,
        Meta::Skip => return None,
        Meta::Item(i) => match i {
            Item::Positional {
                metavar,
                strict,
                help: _,
            } => {
                *is_pos = true;
                if *strict {
                    UsageMeta::StrictPos(metavar)
                } else {
                    UsageMeta::Pos(metavar)
                }
            }
            Item::Command {
                name: _,
                help: _,
                short: _,
                meta: _,
            } => {
                *is_pos = true;
                if *had_commands {
                    return None;
                }
                *had_commands = true;
                UsageMeta::Command
            }
            Item::Flag { name, help: _ } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => UsageMeta::ShortFlag(*s),
                ShortLong::Long(l) => UsageMeta::LongFlag(l),
            },
            Item::Argument {
                name,
                metavar,
                env: _,
                help: _,
            } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => {
                    UsageMeta::ShortArg(*s, metavar)
                }
                ShortLong::Long(l) => UsageMeta::LongArg(l, metavar),
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
        use std::fmt::Write;
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
            UsageMeta::Required(x) => write!(f, "({})", x),
            UsageMeta::Optional(x) => write!(f, "[{}]", x),
            UsageMeta::Many(x) => write!(f, "{}...", x),
            UsageMeta::ShortFlag(c) => write!(f, "-{}", c),
            UsageMeta::LongFlag(l) => write!(f, "--{}", l),
            UsageMeta::ShortArg(c, v) => write!(f, "-{} {}", c, v),
            UsageMeta::LongArg(l, v) => write!(f, "--{} {}", l, v),
            UsageMeta::Command => f.write_str("COMMAND ..."),
            UsageMeta::Pos(s) => write!(f, "<{}>", s),
            UsageMeta::StrictPos(s) => write!(f, "-- <{}>", s),
        }
    }
}
