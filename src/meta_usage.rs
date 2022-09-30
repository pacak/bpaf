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
    let mut is_pos = false;

    let usage = collect_usage_meta(meta, &mut is_pos)?;
    if let UsageMeta::Or(..) = &usage {
        Some(UsageMeta::Required(Box::new(usage)))
    } else {
        Some(usage)
    }
}
/// Transforms `Meta` to [`UsageMeta`]
///
/// return value is None if parser takes no parameters at all
///
/// 1. OR construct with 1 item is just one item
/// 2. AND construct with 1 item is just one item
/// 3. Flatten Commands inside OR constructs
/// 4  Set required for top level
/// 5. Set required for Items inside OR and AND constructs
/// 6. Set optional for Items marked inside [`Meta::Optional`]
/// 7. put optional OR construct into [`UM::Optional`]
/// 8. put required OR constructs into [`UM::Required`]
/// 9. put [] around any optional item
/// 10. put () around AND construct and commands inside [`Meta::Many`]
fn collect_usage_meta(meta: &Meta, is_pos: &mut bool) -> Option<UsageMeta> {
    let r = match meta {
        Meta::And(xs) => {
            let mut items = xs
                .iter()
                .filter_map(|x| {
                    let mut this_pos = false;
                    let usage_meta = collect_usage_meta(x, &mut this_pos)?;
                    assert!(!*is_pos || this_pos,
                        "bpaf usage BUG: all positional and command items must be placed in the right \
                        most position of the structure or tuple they are in but {} breaks this rule. \
                        See bpaf documentation for `positional` and `positional_os` for details.",
                        usage_meta
                    );
                    *is_pos |= this_pos;

                    if let UsageMeta::Or(_) = &usage_meta {
                        Some(UsageMeta::Required(Box::new(usage_meta)))
                    } else {
                    Some(usage_meta)
                    }
                })
                .collect::<Vec<_>>();
            match items.len() {
                0 => return None,
                1 => items.remove(0),
                _ => UsageMeta::And(items),
            }
        }

        Meta::Or(xs) => {
            let mut saw_command = false;
            let mut any_pos = false;
            let mut items = xs
                .iter()
                .filter_map(|x| {
                    let mut top_pos = *is_pos;
                    let usage_meta = collect_usage_meta(x, &mut top_pos)?;
                    any_pos |= top_pos;
                    if let UsageMeta::Command = &usage_meta {
                        if saw_command {
                            None
                        } else {
                            saw_command = true;
                            Some(usage_meta)
                        }
                    } else {
                        Some(usage_meta)
                    }
                })
                .collect::<Vec<_>>();
            *is_pos |= any_pos;
            match items.len() {
                0 => return None,
                1 => items.remove(0),
                _ => UsageMeta::Or(items),
            }
        }
        Meta::Optional(m) => {
            let inner = collect_usage_meta(m, is_pos)?;
            UsageMeta::Optional(Box::new(inner))
        }
        Meta::Many(meta) => {
            let inner = collect_usage_meta(meta, is_pos)?;
            if let UsageMeta::And(..) | UsageMeta::Or(..) = &inner {
                UsageMeta::Many(Box::new(UsageMeta::Required(Box::new(inner))))
            } else {
                UsageMeta::Many(Box::new(inner))
            }
        }
        Meta::Decorated(meta, _) => collect_usage_meta(meta, is_pos)?,
        Meta::Skip => return None,
        Meta::Item(i) => match i {
            Item::Positional {
                metavar, strict, ..
            } => {
                *is_pos = true;
                if *strict {
                    UsageMeta::StrictPos(metavar)
                } else {
                    UsageMeta::Pos(metavar)
                }
            }
            Item::Command { .. } => {
                *is_pos = true;
                UsageMeta::Command
            }
            Item::Flag { name, .. } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => UsageMeta::ShortFlag(*s),
                ShortLong::Long(l) => UsageMeta::LongFlag(l),
            },
            Item::Argument { name, metavar, .. } => match name {
                ShortLong::Short(s) | ShortLong::ShortLong(s, _) => {
                    UsageMeta::ShortArg(*s, metavar)
                }
                ShortLong::Long(l) => UsageMeta::LongArg(l, metavar),
            },
        },
    };
    Some(r)
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
