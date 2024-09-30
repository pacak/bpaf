#[derive(Debug, Clone, Default)]
struct Usage<'a> {
    events: Vec<Event<'a>>,
    group_start: Vec<usize>,
}

#[derive(Debug, Copy, Clone)]
enum Event<'a> {
    Item(&'a Item),
    Command,
    Text(&'a str),
    And {
        items: usize,
        required: bool,
    },
    Or {
        items: usize,
        required: bool,
        has_command: bool,
    },
    Optional,
    Many,

    GroupEnd {
        index: usize,
    },
}

impl Event<'_> {
    fn as_group_start(&self) -> Option<usize> {
        if let Event::GroupEnd { index } = self {
            Some(*index)
        } else {
            None
        }
    }
    fn as_group_start_mut(&mut self) -> Option<&mut usize> {
        if let Event::GroupEnd { index } = self {
            Some(index)
        } else {
            None
        }
    }
    fn is_group_ty(&self, ty: GroupTy) -> bool {
        matches!(
            (self, ty),
            (Event::And { .. }, GroupTy::And) | (Event::Or { .. }, GroupTy::Or)
        )
    }
}

#[derive(Copy, Clone)]
enum GroupTy {
    And,
    Or,
}

impl Usage<'_> {
    /// collapse groups
    /// - products in products: (-a (-b -c)) => (-a -b -c)
    /// - sums in sums: (-a | (-b | -c)) => (-a | -b | -c)
    fn group_collapse(&mut self) {
        for ty in &[GroupTy::And, GroupTy::Or] {
            // we are going to scan events right to left to make sure to cover
            // cases of multiple nested groups: ((-a) (-b))
            // since scanning left to right will yield us sibling groups
            // (-a) and (-b) first (we match on group end rather than group start),
            // requiring multiple passes.
            let Some((mut prev_start, mut prev_end)) = self.group_before(self.events.len()) else {
                continue;
            };

            // then we start looking for groups, right to left
            while let Some((next_start, next_end)) = self.group_before(prev_end) {
                if prev_start < next_start
                    && next_end < prev_end
                    && self.events[prev_start].is_group_ty(*ty)
                    && self.events[next_start].is_group_ty(*ty)
                {
                    // at this point we know that next_start..next_end is fully inside of
                    // prev_start..prev_end group and it is the same group type so we can drop
                    // the inner group;

                    self.remove_events(next_start, next_end);
                    prev_end -= 2;
                } else {
                    prev_start = next_start;
                    prev_end = next_end;
                }
            }
        }
    }
    /// Remove a pair of events, adjusting affected indices
    fn remove_events(&mut self, a: usize, b: usize) {
        for to_ix in a..self.events.len() - 2 {
            let mut from_ix = to_ix + 1;
            if from_ix >= b {
                from_ix += 1;
            }
            self.events[to_ix] = self.events[from_ix];
            if let Some(m) = self.events[to_ix].as_group_start_mut() {
                if *m >= b {
                    *m -= 2;
                } else if *m >= a {
                    *m -= 1;
                }
            }
        }
        self.events.truncate(self.events.len() - 2);
    }

    #[inline(never)]
    fn group_before(&self, mut cur: usize) -> Option<(usize, usize)> {
        assert!(self.events.len() + 1 > cur);
        while cur > 0 {
            cur -= 1;
            if let Some(start) = self.events[cur].as_group_start() {
                return Some((start, cur));
            }
        }
        None
    }

    #[inline(never)]
    fn group_after(&self, start: usize) -> Option<(usize, usize)> {
        self.events[start..]
            .iter()
            .enumerate()
            .find_map(|(end, e)| Some((e.as_group_start()?, end + start)))
    }

    /// collapse single element products and sums: (-a) => -a
    fn single_collapse(&mut self) {}
    fn count_items(&self, range: std::ops::Range<usize>) -> usize {
        self.events[range]
            .iter()
            .filter(|i| match i {
                Event::Item(_) | Event::Command | Event::Text(_) => true,
                Event::And { .. }
                | Event::Or { .. }
                | Event::Optional
                | Event::Many
                | Event::GroupEnd { .. } => false,
            })
            .count()
    }

    fn render(mut self) -> String {
        self.group_collapse();
        todo!("{self:?}");
    }
}

impl<'a> Visitor<'a> for Usage<'a> {
    fn item(&mut self, item: &'a Item) {
        self.events.push(Event::Item(item));
    }

    fn command(&mut self, _long_name: &'a str, _short_name: char) -> bool {
        self.events.push(Event::Command);
        false
    }

    fn push(&mut self, decor: Decor) {
        self.group_start.push(self.events.len());
        self.events.push(match decor {
            Decor::Many => Event::Many,
            Decor::Optional => Event::Optional,
            Decor::And => Event::And {
                items: 0,
                required: false,
            },
            Decor::Or => Event::Or {
                items: 0,
                required: false,
                has_command: false,
            },
        })
    }
    fn pop(&mut self) {
        let index = self.group_start.pop().expect("Unbalanced groups!");
        self.events.push(Event::GroupEnd { index });
    }
}

#[test]
fn opt_flag() {
    let mut v = Usage::default();
    {
        let this = &mut v;
        this.push(Decor::Optional);
        this.item(&Item::Flag(ShortLong::Short('v')));
        this.pop();
    };
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

    assert_eq!(Some((0, 5)), u.group_before(6));
    assert_eq!(Some((3, 4)), u.group_before(5));
    assert_eq!(Some((1, 2)), u.group_before(4));
}

/// Contains name for named
#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
pub enum Item {
    Flag(ShortLong),
    Argument(ShortLong, &'static str),
    Positional(&'static str),
}

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
    fn command(&mut self, long_name: &'a str, short_name: char) -> bool;
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
