#![allow(dead_code)]

mod visitor;

use visitor::Visitor;

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

pub mod ex2;
pub mod ex3;
pub mod ex4;

// pub mod ex {
//     use std::{
//         cell::{Cell, RefCell},
//         collections::HashMap,
//         rc::Rc,
//     };
//
//     #[derive(Debug, Copy, Clone)]
//     struct SparkId(usize);
//
//     struct Ctx {
//         next_child: usize,
//         pos: usize,
//         values: Vec<String>,
//         shorts: HashMap<char, Vec<Rc<RefCell<dyn Spark>>>>,
//         longs: HashMap<&'static str, Vec<Rc<RefCell<dyn Spark>>>>,
//         posn: Vec<Rc<dyn Spark>>,
//         sparks: HashMap<SparkId, Box<dyn Spark>>,
//     }
//
//     impl Ctx {
//         fn new(values: Vec<String>) -> Self {
//             Self {
//                 values,
//                 next_child: 0,
//                 pos: 0,
//
//                 posn: Vec::new(),
//                 shorts: HashMap::new(),
//                 longs: HashMap::new(),
//                 sparks: HashMap::new(),
//             }
//         }
//
//         fn take_front(&mut self) -> Option<String> {
//             self.values.get(self.pos).cloned()
//         }
//
//         fn child_id(&mut self) -> SparkId {
//             let x = SparkId(self.next_child);
//             self.next_child += 1;
//             x
//         }
//     }
//
//     // how to detect when to remove a spark after parsing a single item?
//     // - for short('v').flag().many() - parent is many and will consume more items
//     // - for short('v').map(|x| Blah).many() - parent is still eventually many
//     // - for short('v') inside of a construct - parent takes only one item
//     //
//     // when construct finalizes - need to kill
//
//     trait Spark {
//         fn eval(&mut self, ctx: &mut Ctx);
//         fn parent(&self) -> SparkId;
//     }
//
//     struct Arg {
//         out: Rc<Cell<Option<String>>>,
//         parent: SparkId,
//     }
//
//     impl Spark for Arg {
//         fn eval(&mut self, ctx: &mut Ctx) {
//             assert!(self.out.replace(ctx.take_front()).is_none());
//             ctx.pos += 1;
//         }
//
//         fn parent(&self) -> SparkId {
//             self.parent
//         }
//     }
//
//     struct Flag {
//         // seen/not seen
//         out: Rc<Cell<bool>>,
//         parent: SparkId,
//     }
//
//     impl Spark for Flag {
//         fn eval(&mut self, _ctx: &mut Ctx) {
//             self.out.set(true);
//         }
//
//         fn parent(&self) -> SparkId {
//             self.parent
//         }
//     }
//
//     enum X<T> {
//         Pending,
//         Value(T),
//         Done,
//     }
//
//     enum Value {
//         Flag(Rc<Cell<bool>>),
//         Arg(Rc<Cell<String>>),
//     }
//     struct Leaf {
//         parent: SparkId,
//         value: Value,
//     }
//
//     // This new approach is to allow positionals to be intermixed with named
//
//     // HashMap<Name, Vec<Leaf>>
//     // eval:
//     // pick first leaf, run it, notify parent recursively
//     // spark eval can produce one of
//     // - done
//     // - skip
//     // - fallback
//     //
//     //
//     // bits:
//     // - collect / some + catch
//     // - optional + catch
//     // - fallback
//     // - map / parse / guard
//
//     // catch needs to be able to rollback parsing - which is fine as long as
//     // sparks always consume items from chans
//
//     impl Ctx {
//         fn flag(
//             &mut self,
//             parent: SparkId,
//             longs: &[&'static str],
//             shorts: &[char],
//         ) -> Rc<Cell<bool>> {
//             let out = Rc::new(Cell::new(false));
//             let res = out.clone();
//             let spark = Rc::new(RefCell::new(Flag { out, parent }));
//             self.register(longs, shorts, spark);
//             res
//         }
//
//         fn arg(
//             &mut self,
//             parent: SparkId,
//             longs: &[&'static str],
//             shorts: &[char],
//         ) -> Rc<Cell<Option<String>>> {
//             let out = Rc::new(Cell::new(None));
//             let res = out.clone();
//             let spark = Rc::new(RefCell::new(Arg { out, parent }));
//             self.register(longs, shorts, spark);
//             res
//         }
//
//         fn register(
//             &mut self,
//             longs: &[&'static str],
//             shorts: &[char],
//             spark: Rc<RefCell<dyn Spark>>,
//         ) {
//             for &long in longs {
//                 self.longs.entry(long).or_default().push(spark.clone());
//             }
//             for &short in shorts {
//                 self.shorts.entry(short).or_default().push(spark.clone());
//             }
//         }
//
//         fn eval(&mut self) {
//             while let Some(elt) = self.values.get(self.pos) {
//                 if let Some(long) = elt.strip_prefix("--") {
//                     if let Some(spark) = self.longs.get(long).and_then(|x| x.first().cloned()) {
//                         self.pos += 1;
//                         spark.try_borrow_mut().unwrap().eval(self);
//                         let parent = spark.try_borrow_mut().unwrap().parent();
//                         // push parent, see if parent wants to keep it?
//                     }
//                 } else if let Some(short) = elt.strip_prefix("-") {
//                     let short = short.chars().next().unwrap(); // TODO
//                     if let Some(spark) = self.shorts.get(&short).and_then(|x| x.first().cloned()) {
//                         self.pos += 1;
//                         spark.try_borrow_mut().unwrap().eval(self);
//                         let parent = spark.try_borrow_mut().unwrap().parent();
//                     }
//                 } else {
//                     panic!("elt: {elt:?}")
//                 }
//             }
//         }
//     }
//     #[test]
//     fn demo() {
//         let xs = ["-a", "hello", "-b"];
//         let mut ctx = Ctx::new(xs.iter().map(|x| String::from(*x)).collect());
//
//         let root = ctx.child_id();
//         let out_a = ctx.arg(root, &[], &['a']);
//         let out_b = ctx.flag(root, &[], &['b']);
//
//         ctx.eval();
//
//         todo!("{:?}", (out_a.take(), out_b.take()))
//     }
// }
