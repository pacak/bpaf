use crate::{buffer::Doc, item::Item};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum Meta {
    /// All arguments listed in a vector must be present
    And(Vec<Meta>),
    /// One of arguments listed in a vector must be present
    Or(Vec<Meta>),
    /// Arguments are optional and encased in [] when rendered
    Optional(Box<Meta>),
    /// Arguments are requred and encased in () when rendered
    Required(Box<Meta>),
    /// Argumens are required to be adjacent to each other
    Adjacent(Box<Meta>),
    /// Primitive argument as described
    Item(Box<Item>),
    /// Accepts multiple arguments
    Many(Box<Meta>),
    /// Arguments form a subsection with buffer being it's header
    ///
    /// whole set of arguments go into the same section as the first one
    Subsection(Box<Meta>, Box<Doc>),
    /// Buffer is rendered after
    Suffix(Box<Meta>, Box<Doc>),
    /// This item is not rendered in the help message
    Skip,
    /// TODO make it Option<Box<Doc>>
    CustomUsage(Box<Meta>, Box<Doc>),
    /// this meta must be prefixed with -- in unsage group
    Strict(Box<Meta>),
}

// to get std::mem::take to work
impl Default for Meta {
    fn default() -> Self {
        Meta::Skip
    }
}

// Meta::Strict should bubble up to one of 3 places:
// - top level
// - one of "and" elements
// - one of "or" elements
#[derive(Debug, Clone, Copy)]
enum StrictNorm {
    /// starting at the top and looking for Strict inside
    Pull,
    Push,
    /// Already accounted for one, can strip the rest
    Strip,
}

impl StrictNorm {
    fn push(&mut self) {
        match *self {
            StrictNorm::Pull => *self = StrictNorm::Push,
            StrictNorm::Push | StrictNorm::Strip => {}
        }
    }
}

impl Meta {
    /// Used by normalization function to collapse duplicated commands.
    /// It seems to be fine to strip section information but not anything else
    fn is_command(&self) -> bool {
        match self {
            Meta::Item(i) => matches!(i.as_ref(), Item::Command { .. }),
            Meta::Subsection(m, _) => m.is_command(),
            _ => false,
        }
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
                Meta::Adjacent(m) => {
                    if let Some(i) = Meta::first_item(m) {
                        if i.is_pos() {
                            go(m, is_pos, v);
                        } else {
                            let mut inner = false;
                            go(m, &mut inner, v);
                        }
                    }
                }
                Meta::Optional(m)
                | Meta::Required(m)
                | Meta::Many(m)
                | Meta::CustomUsage(m, _)
                | Meta::Subsection(m, _)
                | Meta::Strict(m)
                | Meta::Suffix(m, _) => go(m, is_pos, v),
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
        let mut norm = StrictNorm::Pull;
        m.normalize(for_usage, &mut norm);
        // stip outer () around meta unless inner
        if let Meta::Required(i) = m {
            m = *i;
        }
        if matches!(m, Meta::Or(_)) {
            m = Meta::Required(Box::new(m));
        }
        if matches!(norm, StrictNorm::Push) {
            m = Meta::Strict(Box::new(m));
        }
        m
    }

    /// Used by adjacent parsers since it inherits behavior of the front item
    pub(crate) fn first_item(meta: &Meta) -> Option<&Item> {
        match meta {
            Meta::And(xs) => xs.first().and_then(Self::first_item),
            Meta::Item(item) => Some(item),
            Meta::Skip | Meta::Or(_) => None,
            Meta::Optional(x)
            | Meta::Strict(x)
            | Meta::Required(x)
            | Meta::Adjacent(x)
            | Meta::Many(x)
            | Meta::Subsection(x, _)
            | Meta::Suffix(x, _)
            | Meta::CustomUsage(x, _) => Self::first_item(x),
        }
    }

    /// Normalize meta info for display as usage. Required propagates outwards
    fn normalize(&mut self, for_usage: bool, norm: &mut StrictNorm) {
        fn normalize_vec(
            xs: &mut Vec<Meta>,
            for_usage: bool,
            norm: &mut StrictNorm,
            or: bool,
        ) -> Option<Meta> {
            let mut final_norm = *norm;
            for m in xs.iter_mut() {
                let mut this_norm = *norm;
                m.normalize(for_usage, &mut this_norm);
                let target: &mut StrictNorm = if or { &mut final_norm } else { norm };

                match (*target, this_norm) {
                    (_, StrictNorm::Pull) | (StrictNorm::Strip, _) => {}
                    (StrictNorm::Pull, StrictNorm::Push) => {
                        *m = Meta::Strict(Box::new(std::mem::take(m)));
                        *target = StrictNorm::Strip;
                    }
                    _ => {
                        *target = this_norm;
                    }
                }
            }
            xs.retain(|m| !matches!(m, Meta::Skip));

            *norm = final_norm;

            match xs.len() {
                0 => Some(Meta::Skip),
                1 => Some(xs.remove(0)),
                _ => None,
            }
        }

        match self {
            Meta::And(xs) => {
                if let Some(replacement) = normalize_vec(xs, for_usage, norm, false) {
                    *self = replacement;
                }
            }
            // or should have either () or [] around it
            Meta::Or(xs) => {
                if let Some(replacement) = normalize_vec(xs, for_usage, norm, true) {
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
                m.normalize(for_usage, norm);
                if matches!(**m, Meta::Skip) {
                    // Optional(Skip) => Skip
                    *self = Meta::Skip;
                } else if let Meta::Required(mm) | Meta::Optional(mm) = m.as_mut() {
                    // Optional(Required(m)) => Optional(m)
                    // Optional(Optional(m)) => Optional(m)
                    *m = std::mem::take(mm);
                } else if let Meta::Many(many) = m.as_mut() {
                    // Optional(Many(Required(m))) => Many(Optional(m))
                    if let Meta::Required(x) = many.as_mut() {
                        *self = Meta::Many(Box::new(Meta::Optional(std::mem::take(x))));
                    }
                }
            }
            Meta::Required(m) => {
                m.normalize(for_usage, norm);
                if matches!(**m, Meta::Skip) {
                    // Required(Skip) => Skip
                    *self = Meta::Skip;
                } else if matches!(**m, Meta::And(_) | Meta::Or(_)) {
                    // keep () around composite parsers
                } else {
                    // and strip them elsewhere
                    *self = std::mem::take(m);
                }
            }
            Meta::Many(m) => {
                m.normalize(for_usage, norm);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                }
            }
            Meta::Adjacent(m) | Meta::Subsection(m, _) | Meta::Suffix(m, _) => {
                m.normalize(for_usage, norm);
                *self = std::mem::take(m);
            }
            Meta::Item(i) => i.normalize(for_usage),
            Meta::Skip => {
                // nothing to do with items and skip just bubbles upwards
            }
            Meta::CustomUsage(m, u) => {
                m.normalize(for_usage, norm);
                // strip CustomUsage if we are not in usage so writer can simply render it
                if for_usage {
                    if u.is_empty() {
                        *self = Meta::Skip;
                    }
                } else {
                    *self = std::mem::take(m);
                }
            }
            Meta::Strict(m) => {
                m.normalize(for_usage, norm);
                norm.push();
                *self = std::mem::take(m);
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
                Item::Any { .. } | Item::Positional { .. } => {}
                Item::Command { meta, .. } => {
                    meta.collect_shorts(flags, args);
                }
                Item::Flag { shorts, .. } => flags.extend(shorts),
                Item::Argument { shorts, .. } => args.extend(shorts),
            },
            Meta::CustomUsage(m, _)
            | Meta::Required(m)
            | Meta::Optional(m)
            | Meta::Adjacent(m)
            | Meta::Subsection(m, _)
            | Meta::Suffix(m, _)
            | Meta::Many(m) => {
                m.collect_shorts(flags, args);
            }
            Meta::Skip | Meta::Strict(_) => {}
        }
    }
}
