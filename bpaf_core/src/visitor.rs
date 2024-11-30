use super::Item;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Group {
    // inner parser can succeed multiple times, requred unless made optional
    Many,
    // inner parser can succeed with no input
    Optional,
    // product group, all members must succeed
    And,
    // sum group, exactly one member must succeed
    Or,
}

pub trait Visitor<'a> {
    fn command(&mut self, long_name: &'a str, short_name: Option<char>) -> bool;
    fn item(&mut self, item: &'a Item);
    fn push_group(&mut self, decor: Group);
    fn pop_group(&mut self);
}

mod usage {
    use super::{Group, Item, Visitor};

    #[derive(Debug, Clone, Default)]
    pub(crate) struct Usage<'a> {
        events: Vec<Event<'a>>,
        group_start: Vec<usize>,
    }

    impl Usage<'_> {
        fn parent_ty(&self) -> Option<Group> {
            if let Event::Group { ty, .. } = self.events.get(*self.group_start.last()?)? {
                Some(*ty)
            } else {
                None
            }
        }

        fn siblings_mut(&mut self) -> Option<&mut usize> {
            let offset = *self.group_start.last()?;
            match self.events.get_mut(offset)? {
                Event::Group { children, .. } => Some(children),
                _ => None,
            }
        }

        fn child_behavior(&mut self, behav: Behav) -> Option<&mut usize> {
            let offset = *self.group_start.last()?;
            match self.events.get_mut(offset)? {
                Event::Group {
                    ty: Group::And,
                    behavior,
                    children,
                } => {
                    *behavior = (*behavior).min(behav);
                    Some(children)
                }
                Event::Group {
                    ty: Group::Or,
                    behavior,
                    children,
                } => {
                    *behavior = (*behavior).max(behav);
                    Some(children)
                }
                // even if child insta fails to parse - optional/many
                // will still succeed
                Event::Group {
                    ty: Group::Optional | Group::Many,
                    children,
                    ..
                } => Some(children),
                _ => None,
            }
        }

        fn render(self) -> String {
            todo!("{self:?}");
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    enum Event<'a> {
        Item(&'a Item),
        Command,
        Strict,
        Text(&'a str),
        Group {
            ty: Group,
            behavior: Behav,
            children: usize,
        },
        Pop,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    enum Behav {
        Fails,
        Runs,
        Succeeds,
    }

    impl Usage<'_> {
        /// Removes all but one top level command from a given range
        /// returns number of commands removed
        fn dedupe_commands_after(&mut self, start: usize) -> usize {
            let mut from = start + 1;
            let mut to = start + 1;
            let mut seen = 0usize;
            let mut depth = 0;
            while let Some(evt) = self.events.get(from) {
                match evt {
                    Event::Group { .. } => depth += 1,
                    Event::Pop => depth -= 1,
                    Event::Command if depth == 0 => {
                        seen += 1;
                        // if we saw a command at 0th depth
                        // skip all the remaining ones
                        if seen > 1 {
                            from += 2;
                            to += 1;
                            continue;
                        }
                    }
                    _ => {}
                }
                self.events[to] = self.events[from];
                from += 1;
                to += 1;
            }
            let removed = seen.saturating_sub(1);
            self.events.truncate(self.events.len() - removed);
            removed
        }
    }

    impl<'a> Visitor<'a> for Usage<'a> {
        fn item(&mut self, item: &'a Item) {
            self.events.push(Event::Item(item));
            if let Some(siblings) = self.siblings_mut() {
                *siblings += 1;
            }
        }

        fn command(&mut self, _long_name: &'a str, _short_name: Option<char>) -> bool {
            if let Some(siblings) = self.siblings_mut() {
                *siblings += 1;
            }
            self.events.push(Event::Command);
            false
        }

        fn push_group(&mut self, decor: Group) {
            if let Some(siblings) = self.siblings_mut() {
                *siblings += 1;
            }
            self.group_start.push(self.events.len());
            let children = 0;
            let behavior = Behav::Runs;
            self.events.push(Event::Group {
                ty: decor,
                children,
                behavior,
            });
        }

        fn pop_group(&mut self) {
            let open = self.group_start.pop().expect("Unbalanced groups!");
            match self.events[open] {
                Event::Group {
                    ty: Group::And,
                    children,
                    mut behavior,
                } => {
                    if children == 0 {
                        behavior = Behav::Succeeds;
                    }
                    if children <= 1 {
                        self.events.remove(open);
                    } else if behavior == Behav::Fails {
                        self.events.drain(open..);
                    } else if self.parent_ty() == Some(Group::And) {
                        if let Some(sib) = self.siblings_mut() {
                            *sib += children - 1;
                        }
                        self.events.remove(open);
                    } else {
                        self.events.push(Event::Pop);
                    }
                    if let Some(siblings) = self.child_behavior(behavior) {
                        // if a group is an instant fail - we are removing all of its children
                        *siblings -= usize::from(behavior == Behav::Fails || children == 0);
                    }
                }
                Event::Group {
                    ty: Group::Or,
                    mut behavior,
                    mut children,
                } => {
                    children -= self.dedupe_commands_after(open);

                    if children == 0 {
                        behavior = Behav::Fails;
                    }
                    if children <= 1 {
                        self.events.remove(open);
                    } else if behavior == Behav::Fails {
                        self.events.drain(open..);
                    } else if self.parent_ty() == Some(Group::Or) {
                        if let Some(siblings) = self.siblings_mut() {
                            *siblings += children - 1;
                        }
                        self.events.remove(open);
                    } else {
                        self.events.push(Event::Pop);
                    }
                    if let Some(siblings) = self.child_behavior(behavior) {
                        // if a group is an instant fail - we are removing all of its children
                        *siblings -= usize::from(behavior == Behav::Fails || children == 0);
                    }
                    if behavior == Behav::Succeeds {
                        self.events.insert(
                            open,
                            Event::Group {
                                ty: Group::Optional,
                                behavior,
                                children: 1,
                            },
                        );
                        self.events.push(Event::Pop);
                    }
                }
                Event::Group {
                    ty: Group::Optional,
                    behavior,
                    mut children,
                } => {
                    debug_assert!(children <= 1);
                    if behavior == Behav::Fails && children > 0 {
                        // it doesn't matter if items inside .optional group fails
                        self.events.drain(open + 1..);
                        #[allow(unused_assignments)]
                        {
                            children = 0;
                        }
                    }

                    if self.parent_ty() == Some(Group::Or) {
                        self.events.remove(open);
                        self.child_behavior(Behav::Succeeds);
                    } else if self.parent_ty() == Some(Group::Optional) {
                        self.events.remove(open);
                    } else {
                        self.events.push(Event::Pop);
                    }
                }
                Event::Group {
                    ty: Group::Many,
                    behavior,
                    children,
                } => {
                    debug_assert!(children <= 1);
                    if behavior == Behav::Fails {
                        if let Some(siblings) = self.siblings_mut() {
                            *siblings -= 1;
                        }
                        self.events.drain(open..);
                    } else if self.parent_ty() == Some(Group::Many) {
                        self.events.remove(open);
                    } else {
                        self.events.push(Event::Pop);
                    }
                }

                Event::Item(_) | Event::Command | Event::Strict | Event::Text(_) | Event::Pop => {
                    panic!("Unbalanced groups!")
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ShortLong;

        const FLAG_A: Item = Item::Flag(ShortLong::Short('a'));
        const FLAG_B: Item = Item::Flag(ShortLong::Short('b'));
        const FLAG_C: Item = Item::Flag(ShortLong::Short('c'));
        const FLAG_D: Item = Item::Flag(ShortLong::Short('d'));

        #[test]
        fn visit_optional_in_or_group_first() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.push_group(Group::Optional);
            v.item(&FLAG_A);
            v.pop_group();
            v.item(&FLAG_B);
            v.pop_group();
            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Optional,
                        behavior: Behav::Succeeds,
                        children: 1
                    },
                    Event::Group {
                        ty: Group::Or,
                        behavior: Behav::Succeeds,
                        children: 2,
                    },
                    Event::Item(&FLAG_A),
                    Event::Item(&FLAG_B),
                    Event::Pop,
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_optional_in_or_group_second() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.item(&FLAG_A);
            v.push_group(Group::Optional);
            v.item(&FLAG_B);
            v.pop_group();
            v.pop_group();
            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Optional,
                        behavior: Behav::Succeeds,
                        children: 1
                    },
                    Event::Group {
                        ty: Group::Or,
                        behavior: Behav::Succeeds,
                        children: 2,
                    },
                    Event::Item(&FLAG_A),
                    Event::Item(&FLAG_B),
                    Event::Pop,
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_no_dedupe_commands_in_and() {
            let mut v = Usage::default();
            v.push_group(Group::And);
            v.command("long_name", None);
            v.command("long_name", None);
            v.pop_group();
            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::And,
                        behavior: Behav::Runs,
                        children: 2,
                    },
                    Event::Command,
                    Event::Command,
                    Event::Pop
                ]
            );
        }

        #[test]
        /// Or group should contain at most one command in the output
        fn visit_dedupe_commands_in_or1() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.command("long_name", None);
            v.command("long_name", None);
            v.pop_group();
            assert_eq!(v.events, &[Event::Command]);
        }

        #[test]
        fn visit_dedupe_commands_in_or2() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.push_group(Group::Optional);
            v.command("long_name", None);
            v.pop_group();
            v.command("long_name", None);
            v.pop_group();
            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Optional,
                        behavior: Behav::Succeeds,
                        children: 1
                    },
                    Event::Command,
                    Event::Pop,
                ]
            );
        }

        #[test]
        /// Similar to or2, but here commands are encased inside of an And
        /// group, so they are not flattened
        fn visit_dedupe_commands_in_or3() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.push_group(Group::And);
            v.item(&FLAG_A);
            v.command("long_name", None);
            v.pop_group();
            v.command("long_name", None);
            v.pop_group();
            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Or,
                        behavior: Behav::Runs,
                        children: 2
                    },
                    Event::Group {
                        ty: Group::And,
                        behavior: Behav::Runs,
                        children: 2
                    },
                    Event::Item(&FLAG_A),
                    Event::Command,
                    Event::Pop,
                    Event::Command,
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_remove_empty_groups() {
            let mut v = Usage::default();
            v.push_group(Group::And);
            v.pop_group();
            assert_eq!(v.events, &[]);
        }

        #[test]
        fn visit_unpack_singleton_groups() {
            let mut v = Usage::default();
            v.push_group(Group::And);
            v.item(&FLAG_A);
            v.pop_group();
            assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
        }

        #[test]
        fn visit_unpack_singleton_nested_groups_1() {
            let mut v = Usage::default();
            v.push_group(Group::And);
            v.push_group(Group::Or);
            v.item(&FLAG_A);
            v.pop_group();
            v.pop_group();
            assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
        }

        #[test]
        fn visit_unpack_singleton_nested_groups_2() {
            let mut v = Usage::default();
            v.push_group(Group::And);
            v.push_group(Group::Or);
            v.push_group(Group::And);
            v.item(&FLAG_A);
            v.pop_group();
            v.pop_group();
            v.pop_group();
            assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
        }

        #[test]
        fn visit_flatten_nested_or() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.push_group(Group::Or);
            println!("{v:?}");
            v.item(&FLAG_A);
            v.item(&FLAG_B);
            v.pop_group();
            v.push_group(Group::Or);
            v.item(&FLAG_C);
            v.item(&FLAG_D);
            v.pop_group();
            v.pop_group();

            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Or,
                        behavior: Behav::Runs,
                        children: 4
                    },
                    Event::Item(&FLAG_A),
                    Event::Item(&FLAG_B),
                    Event::Item(&FLAG_C),
                    Event::Item(&FLAG_D),
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_flatten_nested_options() {
            let mut v = Usage::default();
            v.push_group(Group::Optional);
            v.push_group(Group::Optional);
            v.item(&FLAG_A);
            v.pop_group();
            v.pop_group();

            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Optional,
                        behavior: Behav::Runs,
                        children: 1
                    },
                    Event::Item(&FLAG_A),
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_flatten_nested_many() {
            let mut v = Usage::default();
            v.push_group(Group::Many);
            v.push_group(Group::Many);
            v.item(&FLAG_A);
            v.pop_group();
            v.pop_group();

            assert_eq!(
                v.events,
                &[
                    Event::Group {
                        ty: Group::Many,
                        behavior: Behav::Runs,
                        children: 1
                    },
                    Event::Item(&FLAG_A),
                    Event::Pop
                ]
            );
        }

        #[test]
        fn visit_trim_redundant_or_commands() {
            let mut v = Usage::default();
            v.push_group(Group::Or);
            v.command("long1", None);
            v.command("long2", None);
            v.pop_group();
            assert_eq!(v.events, &[Event::Command]);
        }

        #[test]
        fn visit_and_group_collapse() {
            let mut u = Usage::default();
            u.push_group(Group::And);
            u.item(&FLAG_A);
            u.push_group(Group::And);
            u.item(&FLAG_B);
            u.item(&FLAG_C);
            u.pop_group();
            u.push_group(Group::Or);
            u.item(&FLAG_D);
            u.pop_group();
            u.pop_group();
            assert_eq!(
                u.events,
                &[
                    Event::Group {
                        ty: Group::And,
                        behavior: Behav::Runs,
                        children: 4
                    },
                    Event::Item(&FLAG_A),
                    Event::Item(&FLAG_B),
                    Event::Item(&FLAG_C),
                    Event::Item(&FLAG_D),
                    Event::Pop,
                ]
            );
        }

        #[test]
        fn visit_group_flatten() {
            let mut u = Usage::default();
            u.push_group(Group::And);
            u.push_group(Group::And);
            u.pop_group();
            u.item(&FLAG_A);
            u.pop_group();
            assert_eq!(u.events, &[Event::Item(&FLAG_A)]);
        }
    }
}

mod help {

    #![allow(dead_code, unused_variables)]
    use std::collections::BTreeMap;

    use super::*;

    const POSS: &str = "Available positional items:";
    const ARGS: &str = "Available options:";
    const CMDS: &str = "Available commands:";

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    enum Sect<'a> {
        Custom(&'a str),
        Header,
        Pos,
        Arg,
        Cmd,
    }

    struct Help<'a> {
        groups: BTreeMap<Sect<'a>, Group<'a>>,
        current_group: Option<&'a str>,
        group_depth: usize,
    }

    #[derive(Default, Debug)]
    struct Group<'a> {
        header: Vec<&'a str>,
        items: Vec<&'a Item>,
    }

    impl<'a> Group<'a> {}

    impl<'a> Visitor<'a> for Help<'a> {
        fn command(&mut self, long_name: &'a str, short_name: Option<char>) -> bool {
            let sect = self.current_group.map_or(Sect::Cmd, Sect::Custom);
            todo!();
            //self.groups.entry(sect).or
        }

        fn item(&mut self, item: &'a Item) {
            todo!()
        }

        fn push_group(&mut self, decor: super::Group) {
            todo!()
        }

        fn pop_group(&mut self) {
            todo!()
        }
    }
}
