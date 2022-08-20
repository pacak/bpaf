use crate::{
    item::Item,
    meta_usage::{collect_usage_meta, UsageMeta},
};

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

    pub(crate) fn poss(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res, Item::is_positional);
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
    /// `None` indicates no parameters - usage line isn't shown
    pub(crate) fn as_usage_meta(&self) -> Option<UsageMeta> {
        let mut had_commands = false;
        collect_usage_meta(self, true, &mut had_commands)
    }
}
