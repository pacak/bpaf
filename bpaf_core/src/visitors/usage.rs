use std::borrow::Cow;

use crate::{Item, Metavar, Named, Nest, VKind, VisitGroup, Visitor};

#[derive(Debug, Default)]
pub struct Usage<'a> {
    events: Vec<Event<'a>>,
    group_start: Vec<usize>,
}

impl Usage<'_> {
    pub(crate) fn render_to(&self, mut out: &mut String) {
        use std::fmt::Write as _;

        use crate::console_writer::Style;
        const L: &str = Style::Literal.ansi();
        const M: &str = Style::Metavar.ansi();
        const T: &str = Style::Text.ansi();
        // separator goes in front of group opening tags and in front of items, unless
        // this is a first group/item in a group or a line
        let mut first = true;
        let mut stack = Vec::<Group>::new();
        let mut sep = " ";
        // Don't draw parens around top a top level product
        let events = match self.events.first() {
            Some(Event::Group(Group {
                group: VisitGroup::Prod,
                optional: false,
                ..
            })) => &self.events[1..self.events.len() - 1],
            _ => &self.events,
        };

        let mut wrote_strict_this_prod = false;
        for event in events.iter() {
            match event {
                Event::Put(put) => {
                    if !first {
                        out.push_str(sep);
                    }
                    first = false;
                    match put {
                        Put::Named { name, meta } => {
                            _ = match name {
                                ShortOrLong::Short(s) => write!(&mut out, "{L}-{s}{T}"),
                                ShortOrLong::Long(l) => write!(&mut out, "{L}--{l}{T}"),
                            };
                            if let Some(meta) = meta {
                                _ = write!(&mut out, "={M}{meta}{T}");
                            }
                        }
                        Put::Pos { meta, strict } => {
                            if *strict && !wrote_strict_this_prod {
                                _ = write!(&mut out, "-- {M}{meta}{T}");
                                wrote_strict_this_prod = true;
                            } else {
                                _ = write!(&mut out, "{M}{meta}{T}");
                            }
                        }
                        Put::Command => {
                            _ = write!(&mut out, "{M}COMMAND{T} ...");
                        }
                        Put::Text { text } => {
                            out.push_str(text);
                        }
                    }
                }
                Event::Group(
                    g @ Group {
                        group,
                        children,
                        optional,
                        visible,
                    },
                ) => {
                    match group {
                        VisitGroup::Many => {
                            // - this is not possible, many will be always sitting in a
                            // product...
                            debug_assert!(*children <= 1, "Many should be a product");
                        }
                        VisitGroup::Optional => {
                            if !optional {
                                if !first {
                                    out.push_str(sep);
                                }
                                out.push('[');
                                first = true;
                            }
                        }
                        VisitGroup::Prod => {
                            if !first {
                                out.push_str(sep);
                            }
                            if *visible {
                                out.push(if *optional { '[' } else { '(' });
                            }
                            first = true;
                        }
                        VisitGroup::Sum => {
                            if *visible {
                                // Like Prod/Optional: emit separator before the opening
                                // bracket if this is not the first element in the parent,
                                // then mark `first = false` so items inside don't get
                                // the Sum's `" | "` separator prepended.
                                if !first {
                                    out.push_str(sep);
                                }
                                out.push(if *optional { '[' } else { '(' });
                                first = true;
                            }
                        }
                        VisitGroup::Global => {
                            // transparent in usage
                        }
                    }
                    stack.push(*g);
                    sep = get_sep(&stack);
                }
                Event::Pop => {
                    use VisitGroup as VG;

                    let g = stack.pop().unwrap();

                    match g.group {
                        VG::Many => {
                            if g.children > 1 {
                                out.push(')');
                            }
                            out.push_str("...");
                        }
                        VG::Optional => {
                            if !g.optional {
                                out.push(']');
                            }
                        }
                        VG::Prod => {
                            wrote_strict_this_prod = false;
                            if g.visible {
                                out.push(if g.optional { ']' } else { ')' });
                            }
                        }
                        VG::Sum => {
                            if g.visible {
                                out.push(if g.optional { ']' } else { ')' });
                            }
                        }
                        VG::Global => {
                            // transparent in usage
                        }
                    }
                    sep = get_sep(&stack);
                }
            }
        }
    }

    fn siblings_mut(&mut self) -> Option<&mut usize> {
        let offset = *self.group_start.last()?;
        match self.events.get_mut(offset)? {
            Event::Group(g) => Some(&mut g.children),
            _ => None,
        }
    }
}

fn get_sep(stack: &[Group]) -> &'static str {
    stack
        .iter()
        .rev()
        .find_map(|g| match g.group {
            VisitGroup::Many | VisitGroup::Optional | VisitGroup::Global => None,
            VisitGroup::Prod => Some(" "),
            VisitGroup::Sum => Some(" | "),
        })
        .unwrap_or(" ")
}

impl<'a> Visitor<'a> for Usage<'a> {
    fn item(&mut self, item: Item<'a>) {
        if let Some(siblings) = self.siblings_mut() {
            *siblings += 1;
        }
        let put = match item {
            Item::Flag { named } => match ShortOrLong::from_named(named) {
                Some(name) => Put::Named { name, meta: None },
                None => return,
            },
            Item::Arg { named, meta } => match ShortOrLong::from_named(named) {
                Some(name) => Put::Named {
                    name,
                    meta: Some(meta),
                },
                None => return,
            },
            Item::Positional {
                meta,
                help: _,
                strict,
            } => Put::Pos { meta, strict },
            Item::Command { .. } => Put::Command,
            Item::Nested { outer, inner } => {
                match outer {
                    Nest::Named(flag) => {
                        let Some(name) = ShortOrLong::from_named(&flag.named) else {
                            return;
                        };
                        let named = Put::Named { name, meta: None };
                        self.events.push(Event::Put(named));
                    }
                    Nest::Keyword(_) => {
                        self.events.push(Event::Put(Put::Command));
                        return;
                    }
                }
                self.events.push(Event::Put(Put::Text {
                    text: Cow::Borrowed("{"),
                }));
                let mut u = Usage::default();
                inner.vi(&mut u);
                let mut inner_usage = String::new();
                u.render_to(&mut inner_usage);
                self.events.push(Event::Put(Put::Text {
                    text: Cow::Owned(inner_usage),
                }));
                self.events.push(Event::Put(Put::Text {
                    text: Cow::Borrowed("}"),
                }));
                return;
            }
            Item::OptionParser { inner, info: _ } => {
                inner.vi(self);
                return;
            }
            Item::Section {
                title: _,
                descr: _,
                inner,
            } => {
                // Section (group_help) is transparent for usage, undo the sibling count increment
                if let Some(siblings) = self.siblings_mut() {
                    *siblings -= 1;
                }
                inner.vi(self);
                return;
            }
            Item::Rendered { text, gr: _ } => Put::Text {
                text: Cow::Borrowed(text),
            },
        };
        self.events.push(Event::Put(put))
    }

    fn identify(&self) -> crate::VKind {
        VKind::Usage
    }

    fn push_group(&mut self, group: VisitGroup) {
        if let Some(siblings) = self.siblings_mut() {
            *siblings += 1;
        }
        self.group_start.push(self.events.len());
        let mut g = Group {
            group,
            children: 0,
            optional: false,
            visible: true,
        };
        if let Some(Event::Group(parent)) = self.events.last() {
            use VisitGroup as VG;
            #[expect(clippy::single_match, reason = "I expect to add more rules")]
            match (parent.group, g.group) {
                (VG::Optional, VG::Many) => {
                    g.optional = true;
                }
                _ => {}
            }
        }
        self.events.push(Event::Group(g));
    }

    // rules:
    // 1. Option is dropped from children of Sum and placed on Sum itself
    // 2. Anything with no children is dropped, parent qty is reduced
    // 3. Prod and Sum tags with one item are dropped
    // 4. Prod (Sum) nested immediately into Prod (Sum) is dropped
    // 5. Retain only one command per sum or prod
    // 6. replace Optional Prod (Sum) with Prod (Sum) { optional: true }
    // 7. replace Optional of Optional with a single optional layer
    //
    // Many and Optional can have only one child.

    fn pop_group(&mut self) {
        use VisitGroup as VG;
        let open = self.group_start.pop().expect("Unbalanced groups!");

        // remove all but the first command from a SUM - otherwise
        // they will be displayed as "COMMAND | COMMAND | COMMAND ... " which is not helpful
        if let Some(group) = self.events[open].as_group()
            && group.group == VisitGroup::Sum
        {
            let mut commands = 0;
            let removed = self
                .events
                .extract_if(open + 1.., |e| {
                    if matches!(e, Event::Put(Put::Command)) {
                        commands += 1;
                        commands > 1
                    } else {
                        false
                    }
                })
                .count();
            if removed > 0 {
                self.events[open].as_group().unwrap().children -= removed;
            }
        }

        if let Some(parent) = self.group_start.last() {
            let [parent, child] = self
                .events
                .get_disjoint_mut([*parent, open])
                .unwrap()
                .map(|i| i.as_group().unwrap());

            let keep = match (parent.group, child.group) {
                (_, VG::Global) => {
                    // Global is transparent - its children belong to the parent
                    parent.children += child.children - 1;
                    false
                }
                (VG::Global, _) => {
                    // parent is Global, child's items should propagate up
                    parent.children += child.children - 1;
                    parent.optional = child.optional;
                    false
                }
                _ if child.children == 0 => {
                    parent.children -= 1;
                    false
                }
                (VG::Sum, VG::Optional) => {
                    // rule 1
                    parent.optional = true;
                    false
                }
                (VG::Prod, VG::Prod) | (VG::Sum, VG::Sum) => {
                    parent.children += child.children - 1;
                    false
                }
                // Many keeps everything visible - each layer carries distinct
                // semantics that should be rendered independently.
                (VG::Many, VG::Many) => true, // nested repetitions: each renders its own `...`
                (VG::Many, VG::Optional) => true, // `.many()` internal `[xxx]...` pattern
                (VG::Many, VG::Prod | VG::Sum) => {
                    // When Many wraps Sum/Prod and Many itself carries the optional
                    // flag propagate it to the inner group
                    // so [(xxx)...] renders as [xxx]...
                    child.optional |= parent.optional;
                    // repeated group: `(xxx)...` or `[xxx]...`
                    // repeated alternatives: `(xxx | yyy)...`
                    true
                }
                (VG::Optional, VG::Many) => {
                    // `.optional().many()` - outer Optional later
                    //   collapses with Many's internal Optional
                    parent.optional = true;
                    true
                }
                (VG::Optional, VG::Optional) => {
                    // rule 7: collapsed nested optionals render as a single layer
                    // retain childn's optionality in parent
                    parent.optional |= child.optional;
                    false
                }
                // rule 6: Optional + Prod/Sum -> Prod/Sum renders `[...]` instead of `(...)`,
                (VG::Optional, VG::Prod | VG::Sum) => {
                    child.optional = true;
                    parent.optional = true; // suppress Optional's own bracket
                    true
                }
                // Prod keeps everything visible - each is a distinct element inside the product.
                (VG::Prod, VG::Many) => true, // repetition inside product: `(xxx)...`
                (VG::Prod, VG::Optional) => true, // optional item inside product: `[xxx]`
                // Keep Sum visible for genuine alternation; collapse when command
                // dedup has left only one alternative - no brackets needed.
                (VG::Prod, VG::Sum) => child.children > 1,
                // Sum keeps everything visible except Prod, whose brackets are redundant
                // inside the sum's alternation.
                (VG::Sum, VG::Many) => true, // repetition inside alternatives: `(xxx | yyy)...`
                (VG::Sum, VG::Prod) => {
                    child.visible = false; // hide Prod brackets, Sum already wraps in `(xxx | yyy)`
                    true
                }
            };

            if keep {
                self.events.push(Event::Pop)
            } else {
                self.events.remove(open);
            }
        } else {
            let g = self.events[open].as_group().unwrap();
            if (g.group == VG::Sum || g.group == VG::Prod) && g.children == 1 {
                self.events.remove(open);
            } else {
                self.events.push(Event::Pop);
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ShortOrLong<'a> {
    Short(char),
    Long(&'a str),
}

impl<'a> ShortOrLong<'a> {
    fn from_named(named: &'a Named) -> Option<Self> {
        match named.get_short_and_long() {
            (None, None) => None,
            (None, Some(l)) => Some(ShortOrLong::Long(l)),
            (Some(s), None) | (Some(s), Some(_)) => Some(ShortOrLong::Short(s)),
        }
    }
}

#[derive(Debug, Clone)]
enum Put<'a> {
    Named {
        name: ShortOrLong<'a>,
        meta: Option<Metavar>,
    },
    Pos {
        meta: Metavar,
        strict: bool,
    },

    Command,
    Text {
        text: Cow<'a, str>,
    },
}

#[derive(Debug, Copy, Clone)]
struct Group {
    group: VisitGroup,
    children: usize,
    visible: bool,
    /// Indicates that the group is optional rather than mandatory. Wraps contents in [xxx]
    optional: bool,
}

#[derive(Debug, Clone)]
enum Event<'a> {
    Put(Put<'a>),
    Group(Group),
    Pop,
}

impl Event<'_> {
    fn as_group(&mut self) -> Option<&mut Group> {
        match self {
            Event::Group(g) => Some(g),
            _ => None,
        }
    }
}
