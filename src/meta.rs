use crate::{
    buffer::{Buffer, Style},
    item::{Item, ShortLong},
    meta_help::Metavar,
};

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
    Subsection(Box<Meta>, Box<Buffer>),
    /// Buffer is rendered after
    Suffix(Box<Meta>, Box<Buffer>),
    /// This item is not rendered in the help message
    Skip,
    /// TODO make it Option<Box<Buffer>>
    HideUsage(Box<Meta>),
}

// to get std::mem::take to work
impl Default for Meta {
    fn default() -> Self {
        Meta::Skip
    }
}

impl Buffer {
    pub(crate) fn write_shortlong(&mut self, name: &ShortLong) {
        match name {
            ShortLong::Short(s) => {
                self.write_char('-', Style::Literal);
                self.write_char(*s, Style::Literal);
            }
            ShortLong::Long(l) | ShortLong::ShortLong(_, l) => {
                self.write_str("--", Style::Literal);
                self.write_str(l, Style::Literal);
            }
        }
    }
    pub(crate) fn write_metavar(&mut self, mv: Metavar) {
        self.write_str(mv.0, Style::Metavar);
    }

    pub(crate) fn write_item(&mut self, item: &Item) {
        match item {
            Item::Positional {
                anywhere: _,
                metavar,
                strict,
                help: _,
            } => {
                if *strict {
                    self.write_str("-- ", Style::Literal)
                }
                self.write_metavar(*metavar);
            }
            Item::Command {
                name: _,
                short: _,
                help: _,
                meta: _,
                info: _,
            } => {
                self.write_str("COMMAND ...", Style::Metavar);
            }
            Item::Flag {
                name,
                shorts: _,
                env: _,
                help: _,
            } => self.write_shortlong(name),
            Item::Argument {
                name,
                shorts: _,
                metavar,
                env: _,
                help: _,
            } => {
                self.write_shortlong(name);
                self.write_char('=', Style::Text);
                self.write_metavar(*metavar);
            }
        }
    }

    pub(crate) fn write_meta(&mut self, meta: &Meta, for_usage: bool) {
        fn go(meta: &Meta, f: &mut Buffer) {
            match meta {
                Meta::And(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" ", Style::Text);
                        }
                        go(x, f);
                    }
                }
                Meta::Or(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" | ", Style::Text);
                        }
                        go(x, f);
                    }
                }
                Meta::Optional(m) => {
                    f.write_str("[", Style::Text);
                    go(m, f);
                    f.write_str("]", Style::Text)
                }
                Meta::Required(m) => {
                    f.write_str("(", Style::Text);
                    go(m, f);
                    f.write_str(")", Style::Text)
                }
                Meta::Item(i) => f.write_item(i),
                Meta::Many(m) => {
                    go(m, f);
                    f.write_str("...", Style::Text)
                }

                Meta::Adjacent(m) | Meta::Subsection(m, _) | Meta::Suffix(m, _) => go(m, f),
                Meta::Skip => f.write_str("no parameters expected", Style::Text),
                Meta::HideUsage(m) => {
                    // if normalization strips this depending on for_usage flag
                    // TODO use buffer
                    //
                    go(m, f);
                }
            }
        }

        let meta = meta.normalized(for_usage);
        go(&meta, self);
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
                | Meta::HideUsage(m)
                | Meta::Subsection(m, _)
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

    /// Used by adjacent parsers since it inherits behavior of the front item
    pub(crate) fn first_item(meta: &Meta) -> Option<Item> {
        match meta {
            Meta::And(xs) => xs.first().and_then(Self::first_item),
            Meta::Item(item) => Some(*item.clone()),
            Meta::Skip | Meta::Or(_) => None,
            Meta::Optional(x)
            | Meta::Required(x)
            | Meta::Adjacent(x)
            | Meta::Many(x)
            | Meta::Subsection(x, _)
            | Meta::Suffix(x, _)
            | Meta::HideUsage(x) => Self::first_item(x),
        }
    }

    /// Normalize meta info for display as usage. Required propagates outwards
    fn normalize(&mut self, for_usage: bool) {
        fn normalize_vec(xs: &mut Vec<Meta>, for_usage: bool) -> Option<Meta> {
            xs.iter_mut().for_each(|m| m.normalize(for_usage));
            xs.retain(|m| !matches!(m, Meta::Skip));
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
                    // Optional(Skip) => Skip
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
                m.normalize(for_usage);
                if matches!(**m, Meta::Skip) {
                    *self = Meta::Skip;
                }
            }
            Meta::Adjacent(m) | Meta::Subsection(m, _) | Meta::Suffix(m, _) => {
                m.normalize(for_usage);
                *self = std::mem::take(m);
            }
            Meta::Item(i) => i.for_usage(for_usage),
            Meta::Skip => {
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
                Item::Argument { shorts, .. } => args.extend(shorts),
            },
            Meta::HideUsage(m)
            | Meta::Required(m)
            | Meta::Optional(m)
            | Meta::Adjacent(m)
            | Meta::Subsection(m, _)
            | Meta::Suffix(m, _)
            | Meta::Many(m) => {
                m.collect_shorts(flags, args);
            }
            Meta::Skip => {}
        }
    }
}
