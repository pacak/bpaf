#![allow(dead_code)]
// #![no_std]
// use core::alloc::Vec;
#[derive(Debug, Clone, Default)]
pub(crate) struct Usage<'a> {
    events: Vec<Event<'a>>,
    group_start: Vec<usize>,
}

impl Usage<'_> {
    fn parent(&self) -> Option<Event> {
        let offset = *self.group_start.last()?;
        Some(*self.events.get(offset)?)
    }

    fn siblings_mut(&mut self) -> Option<&mut usize> {
        let offset = *self.group_start.last()?;
        match self.events.get_mut(offset)? {
            Event::And { children, .. }
            | Event::Or { children, .. }
            | Event::Optional { children, .. }
            | Event::Many { children, .. } => Some(children),
            _ => None,
        }
    }
    fn child_behavior(&mut self, behav: Behav) -> Option<&mut usize> {
        let offset = *self.group_start.last()?;
        match self.events.get_mut(offset)? {
            Event::And { behavior, children } => {
                *behavior = (*behavior).min(behav);
                Some(children)
            }
            Event::Or { behavior, children } => {
                *behavior = (*behavior).max(behav);
                Some(children)
            }
            // even if child insta fails to parse - optional/many
            // will still succeed
            Event::Optional { children, .. } | Event::Many { children, .. } => Some(children),
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
    And { behavior: Behav, children: usize },
    Or { behavior: Behav, children: usize },
    Optional { behavior: Behav, children: usize },
    Many { behavior: Behav, children: usize },
    Pop,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Behav {
    Fails,
    Runs,
    Succeeds,
}

impl<'a> Visitor<'a> for Usage<'a> {
    fn item(&mut self, item: &'a Item) {
        self.events.push(Event::Item(item));
        if let Some(siblings) = self.siblings_mut() {
            *siblings += 1;
        }
    }

    fn command(&mut self, _long_name: &'a str, _short_name: Option<char>) -> bool {
        // remove duplicate COMMAND events from a group of Or patterns:
        //
        // This replaces things like `(COMMAND | COMMAND | COMMAND)`
        // with a single `COMMAND`
        //
        // while retaining cases where they go sequentially, adjacent commands
        // should remain as `COMMAND COMMAND`

        // no parent => keep the command
        if !self.group_start.last().map_or(false, |&ix| {
            matches!(self.events[ix], Event::Or { .. })
                && self.events[ix..].contains(&Event::Command)
        }) {
            if let Some(siblings) = self.siblings_mut() {
                *siblings += 1;
            }
            self.events.push(Event::Command);
        }
        false
    }

    fn push_group(&mut self, decor: Group) {
        if let Some(siblings) = self.siblings_mut() {
            *siblings += 1;
        }
        self.group_start.push(self.events.len());
        let children = 0;
        let behavior = Behav::Runs;
        self.events.push(match decor {
            Group::Many => Event::Many { children, behavior },
            Group::Optional => Event::Optional { children, behavior },
            Group::And => Event::And { children, behavior },
            Group::Or => Event::Or { children, behavior },
        });
    }

    fn pop_group(&mut self) {
        let open = self.group_start.pop().expect("Unbalanced groups!");
        match self.events[open] {
            Event::And {
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
                } else if matches!(self.parent(), Some(Event::And { .. })) {
                    if let Some(sib) = self.siblings_mut() {
                        *sib += children - 1;
                    }
                    self.events.remove(open);
                } else {
                    self.events.push(Event::Pop);
                }
                if let Some(siblings) = self.child_behavior(behavior) {
                    // if a group is an instant fail - we are removing all of its children
                    *siblings -= usize::from(behavior == Behav::Fails);
                }
            }
            Event::Or {
                mut behavior,
                children,
            } => {
                if children == 0 {
                    behavior = Behav::Fails;
                }
                if children <= 1 {
                    self.events.remove(open);
                } else if behavior == Behav::Fails {
                    self.events.drain(open..);
                } else if matches!(self.parent(), Some(Event::Or { .. })) {
                    if let Some(siblings) = self.siblings_mut() {
                        *siblings += children - 1;
                    }
                    self.events.remove(open);
                } else {
                    self.events.push(Event::Pop);
                }
                if let Some(siblings) = self.child_behavior(behavior) {
                    // if a group is an instant fail - we are removing all of its children
                    *siblings -= usize::from(behavior == Behav::Fails);
                }
            }
            Event::Optional { behavior, children } => {
                debug_assert!(children <= 1);
                if behavior == Behav::Fails {
                    if let Some(siblings) = self.siblings_mut() {
                        *siblings -= 1;
                    }
                    self.events.drain(open..);
                } else if matches!(self.parent(), Some(Event::Optional { .. })) {
                    self.events.remove(open);
                } else {
                    self.events.push(Event::Pop);
                }
            }
            Event::Many { behavior, children } => {
                debug_assert!(children <= 1);
                if behavior == Behav::Fails {
                    if let Some(siblings) = self.siblings_mut() {
                        *siblings -= 1;
                    }
                    self.events.drain(open..);
                } else if matches!(self.parent(), Some(Event::Many { .. })) {
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

const FLAG_A: Item = Item::Flag(ShortLong::Short('a'));
const FLAG_B: Item = Item::Flag(ShortLong::Short('b'));
const FLAG_C: Item = Item::Flag(ShortLong::Short('c'));
const FLAG_D: Item = Item::Flag(ShortLong::Short('d'));

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
            Event::And {
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
fn visit_dedupe_commands_in_or() {
    let mut v = Usage::default();
    v.push_group(Group::Or);
    v.command("long_name", None);
    v.command("long_name", None);
    v.pop_group();
    assert_eq!(v.events, &[Event::Command,]);
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
            Event::Or {
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
            Event::Optional {
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
            Event::Many {
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
fn opt_flag() {
    let mut v = Usage::default();

    v.push_group(Group::Optional);
    v.item(&Item::Flag(ShortLong::Short('v')));
    v.pop_group();

    assert_eq!(v.render(), "[-v]");
}

#[test]
fn xxx() {
    let mut u = Usage::default();
    u.push_group(Group::And);
    u.push_group(Group::Optional);
    u.item(&Item::Flag(ShortLong::Short('v')));
    u.pop_group();
    u.item(&Item::Positional("FILE"));
    u.pop_group();

    assert_eq!(u.render(), "[-v] FILE");
}

#[test]
fn group_collapse() {
    let mut u = Usage::default();
    u.push_group(Group::And);
    u.item(&Item::Flag(ShortLong::Short('a')));
    u.push_group(Group::And);
    u.item(&Item::Flag(ShortLong::Short('b')));
    u.item(&Item::Flag(ShortLong::Short('c')));
    u.pop_group();
    u.push_group(Group::Or);
    u.pop_group();
    u.pop_group();
    assert_eq!(u.events, &[]);
}

#[test]
fn group_before() {
    let mut u = Usage::default();
    u.push_group(Group::And);
    u.push_group(Group::And);
    u.pop_group();
    u.push_group(Group::And);
    u.pop_group();
    u.pop_group();

    todo!("{u:?}");
    //    assert_eq!(Some((0, 5)), u.group_before(6));
    //    assert_eq!(Some((3, 4)), u.group_before(5));
    //    assert_eq!(Some((1, 2)), u.group_before(4));
}

/// Contains name for named
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShortLong {
    /// Short name only (one char),
    /// Ex `-v` is stored as Short('v'),
    Short(char),
    /// Long name only, could be one char
    Long(&'static str),
    Both(char, &'static str),
}

impl ShortLong {
    pub(crate) fn as_short(&self) -> Self {
        match self {
            ShortLong::Short(s) | ShortLong::Both(s, _) => Self::Short(*s),
            ShortLong::Long(_) => *self,
        }
    }
}
impl std::fmt::Display for ShortLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortLong::Short(s) | ShortLong::Both(s, _) => write!(f, "-{s}"),
            ShortLong::Long(l) => write!(f, "--{l}"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Item {
    Flag(ShortLong),
    Argument(ShortLong, &'static str),
    Positional(&'static str),
}

#[derive(Copy, Clone)]
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

pub trait Parser<T> {
    fn eval(&self, args: &mut State) -> Result<T, Error>;
    fn meta(&self, visitor: &mut dyn Visitor);

    // - usage
    // - documentation and --help
    // -parsing
    // - invariant checking
    // - get available options for errors
}

pub struct State;
pub struct Error;
pub struct Con<E, M> {
    pub eval: E,
    pub meta: M,
    pub failfast: bool,
}

impl<T, E, M> Parser<T> for Con<E, M>
where
    E: Fn(bool, &mut State) -> Result<T, Error>,
    M: Fn(&mut dyn Visitor),
{
    fn eval(&self, args: &mut State) -> Result<T, Error> {
        (self.eval)(self.failfast, args)
    }

    fn meta(&self, visitor: &mut dyn Visitor) {
        (self.meta)(visitor)
    }
}
