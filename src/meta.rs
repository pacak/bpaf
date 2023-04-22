use crate::item::Item;

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub enum DecorPlace {
    Header,
    Suffix,
}
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Meta {
    And(Vec<Meta>),
    Or(Vec<Meta>),
    Optional(Box<Meta>),
    Required(Box<Meta>),
    Anywhere(Box<Meta>),
    Item(Box<Item>),
    Many(Box<Meta>),
    Decorated(Box<Meta>, String, DecorPlace),
    Skip,
    HideUsage(Box<Meta>),
}

impl Default for Meta {
    fn default() -> Self {
        Meta::Skip
    }
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn go(meta: &Meta, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match meta {
                Meta::And(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" ")?;
                        }
                        go(x, f)?;
                    }
                    Ok(())
                }
                Meta::Or(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" | ")?;
                        }
                        go(x, f)?;
                    }
                    Ok(())
                }
                Meta::Optional(m) => {
                    f.write_str("[")?;
                    go(m, f)?;
                    f.write_str("]")
                }
                Meta::Required(m) => {
                    f.write_str("(")?;
                    go(m, f)?;
                    f.write_str(")")
                }
                Meta::Item(i) => i.fmt(f),
                Meta::Many(m) => {
                    go(m, f)?;
                    f.write_str("...")
                }

                // hmm... Do I want to use special syntax here?
                Meta::Anywhere(m) => m.fmt(f),
                Meta::Decorated(m, _, _) => m.fmt(f),
                Meta::Skip => f.write_str("no parameters expected"),
                Meta::HideUsage(m) => {
                    if !f.alternate() {
                        go(m, f)?;
                    }
                    Ok(())
                }
            }
        }

        let meta = self.normalized(f.alternate());
        go(&meta, f)
    }
}

impl Meta {
    fn is_command(&self) -> bool {
        if let Meta::Item(item) = self {
            if let Item::Command { .. } = item.as_ref() {
                return true;
            }
        } else if let Meta::Decorated(m, _, _) = self {
            return m.is_command();
        }

        false
    }

    /// do a nested invariant check
    pub(crate) fn positional_invariant_check(&self, verbose: bool) {
        fn go(meta: &Meta, is_pos: &mut bool, v: bool) {
            match meta {
                Meta::And(xs) => {
                    for x in xs {
                        go(x, is_pos, v);
                    }
                }
                Meta::Or(xs) => {
                    let mut out = *is_pos;
                    for x in xs {
                        let mut this_pos = *is_pos;
                        go(x, &mut this_pos, v);
                        out |= this_pos;
                    }
                    *is_pos = out;
                }
                Meta::Item(i) => {
                    match (*is_pos, i.is_pos()) {
                        (true, true) | (false, false) => {}
                        (true, false) => {
                            panic!("bpaf usage BUG: all positional and command items must be placed in the right \
                        most position of the structure or tuple they are in but {:?} breaks this rule. \
                        See bpaf documentation for `positional` for details.", i);
                        }
                        (false, true) => {
                            *is_pos = true;
                        }
                    }
                    if let Item::Command { meta, .. } = &**i {
                        let mut command_pos = false;
                        if v {
                            println!("Checking\n{:#?}", meta);
                        }
                        go(meta, &mut command_pos, v);
                    }
                }
                Meta::Anywhere(m) => {
                    let mut inner = false;
                    go(m, &mut inner, v);
                }
                Meta::Optional(m)
                | Meta::Required(m)
                | Meta::Many(m)
                | Meta::HideUsage(m)
                | Meta::Decorated(m, _, _) => go(m, is_pos, v),
                Meta::Skip => {}
            }
        }
        let mut is_pos = false;
        if verbose {
            println!("Checking\n{:#?}", self);
        }
        go(self, &mut is_pos, verbose);
    }

    pub(crate) fn normalized(&self, for_usage: bool) -> Meta {
        let mut m = self.clone();
        m.normalize(for_usage);
        // stip outer () around meta unless inner
        if let Meta::Required(i) = m {
            m = *i;
        }
        if matches!(m, Meta::Or(_)) {
            m = Meta::Required(Box::new(m));
        }
        m
    }

    /// Normalize meta info for display as usage. Required propagates outwards
    fn normalize(&mut self, for_usage: bool) {
        fn normalize_vec(xs: &mut Vec<Meta>, for_usage: bool) -> Option<Meta> {
            xs.retain_mut(|m| {
                m.normalize(for_usage);
                !matches!(m, Meta::Skip)
            });
            match xs.len() {
                0 => Some(Meta::Skip),
                1 => Some(xs.remove(0)),
                _ => None,
            }
        }

        match self {
            Meta::And(xs) => {
                if let Some(replacement) = normalize_vec(xs, for_usage) {
                    *self = replacement;
                }
            }
            // or should have either () or [] around it
            Meta::Or(xs) => {
                if let Some(replacement) = normalize_vec(xs, for_usage) {
                    *self = replacement;
                } else {
                    let mut saw_cmd = false;
                    // drop all the commands apart from the first one
                    xs.retain(|m| {
                        let is_cmd = m.is_command();
                        let keep = !(is_cmd && saw_cmd);
                        saw_cmd |= is_cmd;
                        keep
                    });
                    match xs.len() {
                        0 => *self = Meta::Skip,
                        1 => *self = xs.remove(0),
                        _ => *self = Meta::Required(Box::new(std::mem::take(self))),
                    }
                }
            }
            Meta::Optional(m) => {
                m.normalize(for_usage);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                } else if let Meta::Required(mm) | Meta::Optional(mm) = m.as_mut() {
                    // Optional(Required(m)) => Optional(m)
                    // Optional(Optional(m)) => Optional(m)
                    *m = std::mem::take(mm);
                }
            }
            Meta::Required(m) => {
                m.normalize(for_usage);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                } else if matches!(**m, Meta::And(_) | Meta::Or(_)) {
                    // keep () around composite parsers
                } else {
                    // Required(Required(m)) => Required(m)
                    // Required(Optional(m)) => Optional(m)
                    *self = std::mem::take(m);
                }
            }
            Meta::Many(m) => {
                m.normalize(for_usage);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                }
            }
            Meta::Decorated(m, _, _) => {
                m.normalize(for_usage);
                *self = std::mem::take(m);
            }
            Meta::Anywhere(m) => {
                m.normalize(for_usage);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                }
            }
            Meta::Item(_) | Meta::Skip => {
                // nothing to do with items and skip just bubbles upwards
            }
            Meta::HideUsage(m) => {
                m.normalize(for_usage);
                if for_usage || matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                }
            }
        }
    }
}

impl From<Item> for Meta {
    fn from(value: Item) -> Self {
        Meta::Item(Box::new(value))
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

    /// collect different kinds of short names for disambiguation
    pub(crate) fn collect_shorts(&self, flags: &mut Vec<char>, args: &mut Vec<char>) {
        match self {
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    x.collect_shorts(flags, args);
                }
            }
            Meta::Item(m) => match &**m {
                Item::Positional { .. } | Item::Command { .. } => {}
                Item::Flag { shorts, .. } => flags.extend(shorts),
                Item::MultiArg { shorts, .. } | Item::Argument { shorts, .. } => {
                    args.extend(shorts);
                }
            },
            Meta::HideUsage(m)
            | Meta::Required(m)
            | Meta::Optional(m)
            | Meta::Anywhere(m)
            | Meta::Many(m)
            | Meta::Decorated(m, _, _) => {
                m.collect_shorts(flags, args);
            }
            Meta::Skip => {}
        }
    }
}
