#![allow(dead_code)]

use std::ops::Range;
#[derive(Debug, Clone, Default)]
struct Usage<'a> {
    events: Vec<Event<'a>>,
    group_start: Vec<usize>,
}

impl Usage<'_> {
    fn parent(&self) -> Option<(usize, Event)> {
        let offset = *self.group_start.last()?;
        Some((offset, *self.events.get(offset)?))
    }

    fn render(self) -> String {
        todo!("{self:?}");
    }
}

#[derive(Debug, Clone, Copy)]
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
impl Event<'_> {
    fn is_group(&self) -> bool {
        matches!(
            self,
            Event::And { .. } | Event::Or { .. } | Event::Optional { .. } | Event::Many { .. }
        )
    }

    fn is_atom(&self) -> bool {
        matches!(
            self,
            Event::Item(_) | Event::Command | Event::Strict | Event::Text(_)
        )
    }
}

// 1. remove any tags around zero items
// 2. remove and/or tags around single item
// 3. drop inner pair of nested Optional

fn children<'a>(events: &'a [Event]) -> Children<'a> {
    debug_assert!(events.is_empty() || events[0].is_group());
    Children {
        depth: 0,
        res: 0,
        events,
        cur: 1,
        open: 1,
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a [Event<'a>];

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.events.get(self.cur)?;
        todo!();
    }
}

struct Children<'a> {
    depth: usize,
    res: usize,
    cur: usize,
    events: &'a [Event<'a>],
    open: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Behav {
    Runs,
    Fails,
    Succeeds,
}
enum Cnt {
    Zero,
    One,
    Many,
}

fn immediate_children(events: &[Event]) -> usize {
    let mut depth = 0;
    let mut res = 0;

    for event in events {
        if event.is_group() {
            depth += 1;
        } else if matches!(event, Event::Pop) {
            depth -= 1;
        } else if depth == 1 {
            res += 1;
        }
    }
    res
}

impl<'a> Visitor<'a> for Usage<'a> {
    fn item(&mut self, item: &'a Item) {
        self.events.push(Event::Item(item));
    }

    fn command(&mut self, _long_name: &'a str, _short_name: Option<char>) -> bool {
        // remove duplicate COMMAND events from a group of or patterns:
        //
        // This replaces things like `(COMMAND | COMMAND | COMMAND)`
        // with a single `COMMAND`
        //
        // while retaining cases where they go sequentially, adjacent commands
        // should remain as `COMMAND COMMAND`
        let keep = match self.parent() {
            Some((ix, Event::Or)) => !self.events[ix..].contains(&Event::Command),
            _ => true,
        };
        if keep {
            self.events.push(Event::Command);
        }
        false
    }

    fn push(&mut self, decor: Decor) {
        self.group_start.push(self.events.len());
        self.events.push(match decor {
            Decor::Many => Event::Many,
            Decor::Optional => Event::Optional,
            Decor::And => Event::And,
            Decor::Or => Event::Or,
        });
    }

    fn pop(&mut self) {
        let open = self.group_start.pop().expect("Unbalanced groups!");
        let children = immediate_children(&self.events[open..]);

        match self.events[open] {
            Event::And => {
                if children == 0 {
                    self.events.pop();
                    self.events.push(Event::Success);
                } else if children == 1 {
                    self.events.remove(open);
                } else {
                    // remove all successes and failures
                    // remove whole group if there's any immediate failures
                }
            }
            Event::Or => {
                if children == 0 {
                    self.events.pop();
                    self.events.push(Event::Failure);
                } else if children == 1 {
                    self.events.remove(open);
                } else {
                    // remove all failures and failures
                    // unwrap optional children
                    // mark block as optional if there's any successes
                }
            }
            Event::Many => {
                if children == 0 {
                    self.events.pop();
                } else {
                    debug_assert_eq!(children, 1);
                    // if child is a failure - replace with a failure
                    // if child is a success - replace with a success
                    // if child is Many - squash
                }
            }
            Event::Optional => {
                if children == 0 {
                    self.events.pop();
                } else {
                    debug_assert_eq!(children, 1);
                    // if child is a failure - replace with a failure
                    // if child is a success - replace with a success
                    // if child is Optional - squash, picking
                }
            }

            _ => panic!("unbalanced groups!"),
        }

        if open + 1 == self.events.len() && self.events[open].is_group() {
            // remove all the empty groups
            self.events.pop();
        } else if matches!(self.events[open], Event::Or | Event::And)
            && immediate_children(&self.events[open..]) == 1
        {
            // remove And/Or group that contains only one item
            self.events.remove(open);
        } else if matches!(self.events[open], Event::Many | Event::Optional)
            && self
                .group_start
                .last()
                .map_or(false, |parent| self.events[open] == self.events[*parent])
        {
            // flatten nested option/many
            self.events.remove(open);
        } else {
            println!(
                "{:?} {:?}",
                matches!(self.events[open], Event::Many | Event::Optional),
                self
            );

            todo!("{:?} / {:?}", &self.events[open..], self)
        }

        // Optional<Optional<xxx>> => Optional<xxx>
        // Many<Many<xxx>> => Optional<xxx>
        //
        // Or<A, B, Optional<C>> => Optional<Or<A, B, C>>
    }
}

const FLAG_A: Item = Item::Flag(ShortLong::Short('a'));

#[test]
fn visit_remove_empty_groups() {
    let mut v = Usage::default();
    v.push(Decor::And);
    v.pop();
    assert_eq!(v.events, &[]);
}

#[test]
fn visit_unpack_singleton_groups() {
    let mut v = Usage::default();
    v.push(Decor::And);
    v.item(&FLAG_A);
    v.pop();
    assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
}

#[test]
fn visit_unpack_singleton_nested_groups_1() {
    let mut v = Usage::default();
    v.push(Decor::And);
    v.push(Decor::Or);
    v.item(&FLAG_A);
    v.pop();
    v.pop();
    assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
}

#[test]
fn visit_unpack_singleton_nested_groups_2() {
    let mut v = Usage::default();
    v.push(Decor::And);
    v.push(Decor::Or);
    v.push(Decor::And);
    v.item(&FLAG_A);
    v.pop();
    v.pop();
    v.pop();
    assert_eq!(v.events, &[Event::Item(&FLAG_A)]);
}

#[test]
fn visit_flatten_nested_options() {
    let mut v = Usage::default();
    v.push(Decor::Optional);
    v.push(Decor::Optional);
    v.item(&FLAG_A);
    v.pop();
    v.pop();
    assert_eq!(
        v.events,
        &[Event::Optional, Event::Item(&FLAG_A), Event::Pop]
    );
}

#[test]
fn visit_flatten_nested_many() {
    let mut v = Usage::default();
    v.push(Decor::Many);
    v.push(Decor::Many);
    v.item(&FLAG_A);
    v.pop();
    v.pop();
    assert_eq!(v.events, &[Event::Many, Event::Item(&FLAG_A), Event::Pop]);
}

#[test]
fn visit_trim_redundant_or_commands() {
    let mut v = Usage::default();
    v.push(Decor::Or);
    v.command("long1", None);
    v.command("long2", None);
    v.pop();
    assert_eq!(v.events, &[Event::Command]);
}

#[test]
fn opt_flag() {
    let mut v = Usage::default();

    v.push(Decor::Optional);
    v.item(&Item::Flag(ShortLong::Short('v')));
    v.pop();

    assert_eq!(v.render(), "[-v]");
}

#[test]
fn xxx() {
    let mut u = Usage::default();
    u.push(Decor::And);
    u.push(Decor::Optional);
    u.item(&Item::Flag(ShortLong::Short('v')));
    u.pop();
    u.item(&Item::Positional("FILE"));
    u.pop();

    assert_eq!(u.render(), "[-v] FILE");
}

#[test]
fn group_collapse() {
    let mut u = Usage::default();
    u.push(Decor::And);
    u.item(&Item::Flag(ShortLong::Short('a')));
    u.push(Decor::And);
    u.item(&Item::Flag(ShortLong::Short('b')));
    u.item(&Item::Flag(ShortLong::Short('c')));
    u.pop();
    u.push(Decor::Or);
    u.pop();
    u.pop();
    assert_eq!(u.render(), "-a -b -c");
}

#[test]
fn group_before() {
    let mut u = Usage::default();
    u.push(Decor::And);
    u.push(Decor::And);
    u.pop();
    u.push(Decor::And);
    u.pop();
    u.pop();

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
pub enum Decor {
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
    fn pop(&mut self);
    fn push(&mut self, decor: Decor);
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
