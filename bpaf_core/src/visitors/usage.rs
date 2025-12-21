use crate::{Item, Metavar, Named, VKind, VisitGroup, Visitor};

#[derive(Debug, Default)]
pub struct Usage<'a> {
    events: Vec<Event<'a>>,
    group_start: Vec<usize>,
}

impl Usage<'_> {
    pub(crate) fn render(&self) -> String {
        let mut out = String::new();
        self.render_to(&mut out);
        out
    }

    pub(crate) fn render_to(&self, mut out: &mut String) {
        use std::fmt::Write as _;

        use crate::visitors::help::Style;
        const L: &str = Style::Literal.ansi();
        const M: &str = Style::Metavar.ansi();
        const T: &str = Style::Text.ansi();
        // separator goes in front of group opening tags and in front of items, unless
        // this is a first group/item in a group or a line
        let mut first = true;
        let mut stack = Vec::<Group>::new();
        let mut sep = " ";
        let mut out = String::new();
        // Don't draw parens around top a top level product
        let events = match self.events.first() {
            Some(Event::Group(Group {
                group: VisitGroup::Prod,
                optional: false,
                ..
            })) => &self.events[1..self.events.len() - 1],
            _ => &self.events,
        };

        for event in events.iter() {
            println!("Stack: {stack:?}\nprinting {event:?}\n");
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
                        Put::Pos { meta } => {
                            _ = write!(&mut out, "{M}{meta}{T}");
                        }
                        Put::Command => {
                            _ = write!(&mut out, "{M}COMMAND{T}");
                        }
                        Put::Text { text } => todo!(),
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
                            // TODO - this is not possible, many will be always sitting in a
                            // product...
                            if *children > 1 {
                                todo!();
                                out.push('(');
                            }
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
                                out.push(if *optional { '[' } else { '(' });
                            }
                        }
                    }
                    stack.push(*g);

                    sep = stack
                        .iter()
                        .rev()
                        .find_map(|g| match g.group {
                            VisitGroup::Many | VisitGroup::Optional => None,
                            VisitGroup::Prod => Some(" "),
                            VisitGroup::Sum => Some(" | "),
                        })
                        .unwrap_or(" ");
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
                            if g.visible {
                                out.push(if g.optional { ']' } else { ')' });
                            }
                        }
                        VG::Sum => {
                            if g.visible {
                                out.push(if g.optional { ']' } else { ')' });
                            }
                        }
                    }
                    sep = stack
                        .iter()
                        .rev()
                        .find_map(|g| match g.group {
                            VisitGroup::Many | VisitGroup::Optional => None,
                            VisitGroup::Prod => Some(" "),
                            VisitGroup::Sum => Some(" | "),
                        })
                        .unwrap_or(" ");
                    println!("Choosing a new sep after writing {:?}", out);
                    println!("Stack was {stack:?}, sep = {sep}");
                }
            }
        }
        out
    }

    fn siblings_mut(&mut self) -> Option<&mut usize> {
        let offset = *self.group_start.last()?;
        match self.events.get_mut(offset)? {
            Event::Group(g) => Some(&mut g.children),
            _ => None,
        }
    }
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
            Item::Positional { meta, help: _ } => Put::Pos { meta },
            Item::Command { .. } => Put::Command,
            Item::Nested { named, inner } => todo!(),
            Item::OptionParser { .. } => {
                return;
            }
            Item::Section {
                title,
                descr,
                inner,
            } => todo!(),
            Item::Rendered { text } => Put::Text { text },
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
        self.events.push(Event::Group(Group {
            group,
            children: 0,
            optional: false,
            visible: true,
        }));
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
    // Many and Optional can have only one child

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
                (VG::Many, VG::Many) => true,         // XXX
                (VG::Many, VG::Optional) => true,     // XXX
                (VG::Many, VG::Prod) => true,         // XXX
                (VG::Many, VG::Sum) => true,          // XXX
                (VG::Optional, VG::Many) => true,     // XXX
                (VG::Optional, VG::Optional) => true, // XXX
                (VG::Optional, VG::Prod | VG::Sum) => {
                    child.optional = true;
                    parent.optional = true; // TODO use hidden
                    true
                }
                (VG::Prod, VG::Many) => true,     // XXX
                (VG::Prod, VG::Optional) => true, // XXX
                (VG::Prod, VG::Sum) => true,      // XXX
                (VG::Sum, VG::Many) => true,      // XXX
                (VG::Sum, VG::Prod) => {
                    child.visible = false;
                    true // XXX
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

#[derive(Debug, Copy, Clone)]
enum Put<'a> {
    Named {
        name: ShortOrLong<'a>,
        meta: Option<Metavar>,
    },
    Pos {
        meta: Metavar,
    },

    Command,
    Text {
        text: &'a str,
    },
}

#[derive(Debug, Copy, Clone)]
struct Group {
    group: VisitGroup,
    children: usize,
    visible: bool,
    optional: bool,
}

#[derive(Debug, Copy, Clone)]
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
#[cfg(test)]
mod tests {
    use crate::{
        Parser, Visited, construct, long, positional, pure, short,
        visitors::{
            help::{Colorscheme, apply_style},
            usage::Usage,
        },
    };

    fn usage(visited: &impl Visited) -> String {
        let mut u = Usage::default();
        visited.visit(&mut u);
        let out = u.render();

        apply_style(&out, &Colorscheme::DULL, true)
    }

    #[test]
    fn usage_product() {
        let a = short('a').switch();
        let b = short('b').req_flag(());
        let c = long("cat").argument::<usize>("KET").many();
        let d = positional::<usize>("LEN").optional();
        let parser = construct!(a, b, c, d).to_options();

        let r = usage(&parser);
        assert_eq!(r, "[-a] -b [--cat=KET]... [LEN]");
    }

    #[test]
    fn many_and_arg() {
        let parser = short('M')
            .argument::<u32>("ARG")
            .help("with help")
            .many()
            .to_options();
        let r = usage(&parser);
        assert_eq!(r, "[-M=ARG]...");
    }

    #[test]
    fn many_and_pos() {
        let parser = positional::<u32>("Ket")
            .help("with help")
            .many()
            .to_options();
        let r = usage(&parser);
        assert_eq!(r, "[<Ket>]...");
    }

    #[test]
    fn some_and_req() {
        let parser = ra().some("some").to_options();
        assert_eq!(usage(&parser), "-a...");
    }

    #[test]
    fn usage_choice_req() {
        let a = short('a').req_flag(());
        let b = short('b').req_flag(());
        let parser = construct!([a, b]).to_options();
        let r = usage(&parser);
        assert_eq!(r, "(-a | -b)");
    }

    fn ra() -> impl Parser<bool> {
        short('a').req_flag(true)
    }

    fn oa() -> impl Parser<bool> {
        short('a').switch()
    }

    fn rb() -> impl Parser<bool> {
        short('b').req_flag(true)
    }

    fn ob() -> impl Parser<bool> {
        short('b').switch()
    }

    fn ca() -> impl Parser<bool> {
        pure(true).to_options().command("a")
    }

    fn cb() -> impl Parser<bool> {
        pure(true).to_options().command("b")
    }

    #[test]
    fn optional_and_sum_1() {
        let parser = construct!([oa(), ob()]).to_options();
        assert_eq!(usage(&parser), "[-a | -b]");
    }

    #[test]
    fn optional_and_sum_2() {
        let parser = construct!([ra(), ob()]).to_options();
        assert_eq!(usage(&parser), "[-a | -b]");
    }

    #[test]
    fn optional_and_sum_3() {
        let parser = construct!([ra(), rb()]).to_options();
        assert_eq!(usage(&parser), "(-a | -b)");
    }

    #[test]
    fn optional_and_sum_4() {
        let parser = construct!([ra(), ob()]).optional().to_options();
        assert_eq!(usage(&parser), "[-a | -b]");
    }

    #[test]
    fn optional_and_sum_5() {
        let parser = construct!([ra(), rb()]).optional().to_options();
        assert_eq!(usage(&parser), "[-a | -b]");
    }

    #[test]
    fn optional_and_prod_1() {
        let parser = construct!(oa(), ob()).to_options();
        assert_eq!(usage(&parser), "[-a] [-b]");
    }

    #[test]
    fn optional_and_prod_2() {
        let parser = construct!(ra(), ob()).to_options();
        assert_eq!(usage(&parser), "-a [-b]");
    }

    #[test]
    fn optional_and_prod_3() {
        let parser = construct!(ra(), rb()).to_options();
        assert_eq!(usage(&parser), "-a -b");
    }

    #[test]
    fn optional_and_prod_4() {
        let parser = construct!(ra(), ob()).optional().to_options();
        assert_eq!(usage(&parser), "[-a [-b]]");
    }

    #[test]
    fn optional_and_prod_5() {
        let parser = construct!(ra(), rb()).optional().to_options();
        assert_eq!(usage(&parser), "[-a -b]");
    }

    #[test]
    fn flatten_prod_left() {
        let ab = construct!(ra(), rb());
        let parser = construct!(ab, ra()).to_options();
        assert_eq!(usage(&parser), "-a -b -a");
    }

    #[test]
    fn flatten_prod_mid() {
        let ab = construct!(ra(), rb());
        let parser = construct!(ra(), ab, rb()).to_options();
        assert_eq!(usage(&parser), "-a -a -b -b");
    }

    #[test]
    fn flatten_prod_right() {
        let ab = construct!(ra(), rb());
        let parser = construct!(ra(), ab).to_options();
        assert_eq!(usage(&parser), "-a -a -b");
    }

    #[test]
    fn dedup_commands_sum_1() {
        let parser = construct!([ca(), cb()]).to_options();
        assert_eq!(usage(&parser), "COMMAND");
    }

    #[test]
    fn dedup_commands_sum_2() {
        let parser = construct!([oa(), ca(), cb()]).to_options();
        assert_eq!(usage(&parser), "[-a | COMMAND]");
    }

    #[test]
    fn dedup_commands_sum_3() {
        let parser = construct!([ca(), oa(), cb()]).to_options();
        assert_eq!(usage(&parser), "[COMMAND | -a]");
    }

    #[test]
    fn dedup_commands_sum_4() {
        let parser = construct!([ca(), cb(), oa()]).to_options();
        assert_eq!(usage(&parser), "[COMMAND | -a]");
    }

    #[test]
    fn dedup_commands_prod_1() {
        let parser = construct!(ca(), cb()).to_options();
        assert_eq!(usage(&parser), "COMMAND COMMAND");
    }

    #[test]
    fn flatten_prod() {
        let a = short('a').switch();
        let b = short('b').req_flag(());
        let c = short('c').switch();
        let ab = construct!(a, b);
        let parser = construct!(ab, c).many().to_options();
        assert_eq!(usage(&parser), "[[-a] -b [-c]]...");
    }

    #[test]
    fn flatten_sum_of_prods_1() {
        let a = construct!(ra(),);
        let b = construct!(rb(),);
        let parser = construct!([a, b]).to_options();
        assert_eq!(usage(&parser), "(-a | -b)");
    }

    #[test]
    fn flatten_sum_of_prods_3() {
        let a = construct!(ra(), oa()).map(|_| true);
        let parser = construct!([a, rb()]).to_options();
        assert_eq!(usage(&parser), "(-a [-a] | -b)");
    }

    #[test]
    fn flatten_prod_of_prods() {
        let a = construct!(ra(), oa());
        let b = construct!(rb(), ob());
        let parser = construct!(a, b).to_options();
        assert_eq!(usage(&parser), "-a [-a] -b [-b]");
    }
}
