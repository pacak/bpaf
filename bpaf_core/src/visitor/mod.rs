use crate::{error::Metavar, named::Name};

pub(crate) mod explain_unparsed;

#[derive(Debug, Copy, Clone)]
pub enum Item<'a> {
    Flag {
        names: &'a [Name<'a>],
        // help: &'a str,
    },
    Arg {
        names: &'a [Name<'a>],
        meta: Metavar,
        // help: &'a str,
    },
    Positional {
        meta: Metavar,
        // help: &'a str,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Group {
    /// inner parser can succeed multiple times, requred unless made optional
    Many,
    /// inner parser can succeed with no input
    Optional,
    /// product group, all members must succeed
    Prod,
    /// sum group, exactly one member must succeed
    Sum,
}

pub trait Visitor<'a> {
    fn item(&mut self, item: Item<'a>);
    fn command(&mut self, names: &[Name]) -> bool;
    fn push_group(&mut self, group: Group);
    fn pop_group(&mut self);
}
