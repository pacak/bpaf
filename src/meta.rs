use crate::{
    item::Item,
    meta_usage::{to_usage_meta, UsageMeta},
};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Meta {
    And(Vec<Meta>),
    Or(Vec<Meta>),
    Optional(Box<Meta>),
    Item(Item),
    Many(Box<Meta>),
    Decorated(Box<Meta>, &'static str),
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

    /// Represent [`Meta`] as [`UsageMeta`]
    ///
    /// `None` indicates no parameters - usage line isn't shown
    pub(crate) fn as_usage_meta(&self) -> Option<UsageMeta> {
        to_usage_meta(self)
    }

    pub(crate) fn collect_shorts(&self, flags: &mut Vec<char>, args: &mut Vec<char>) {
        match self {
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    x.collect_shorts(flags, args);
                }
            }
            Meta::Item(m) => match m {
                Item::Positional { .. } | Item::Command { .. } => {}
                Item::Flag { shorts, .. } => flags.extend(shorts),
                Item::Argument { shorts, .. } => args.extend(shorts),
            },
            Meta::Optional(m) | Meta::Many(m) | Meta::Decorated(m, _) => {
                m.collect_shorts(flags, args)
            }
            Meta::Skip => {}
        }
    }
}
