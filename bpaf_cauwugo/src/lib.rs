macro_rules! pass_flag {
    ($cmd:ident, $field:expr, $value:literal) => {
        if $field {
            $cmd.arg($value);
        }
    };
}

macro_rules! pass_arg {
    ($cmd:ident, $field:expr, $value:literal) => {
        for v in $field {
            $cmd.arg($value).arg(v);
        }
    };
}

macro_rules! pass_req_arg {
    ($cmd:ident, $field:expr, $value:literal) => {
        $cmd.arg($value).arg($field);
    };
}

use bpaf::Parser;
use std::{cell::RefCell, rc::Rc};

/// Stash successfully parsed value in a RefCell
pub fn remember_opt<P, T>(parser: P, cell: &Rc<RefCell<T>>) -> impl Parser<T>
where
    P: Parser<T>,
    T: Clone + 'static,
{
    let cell = cell.clone();
    parser.map(move |val| {
        *cell.borrow_mut() = val.clone();
        val
    })
}

/// Stash successfully parsed value in a RefCell.
///
/// Unlike [`remember_opt`] this one parses a required value
pub fn remember_req<P, T>(parser: P, cell: &Rc<RefCell<Option<T>>>) -> impl Parser<T>
where
    P: Parser<T>,
    T: Clone + 'static,
{
    let cell = cell.clone();
    parser.map(move |val| {
        *cell.borrow_mut() = Some(val.clone());
        val
    })
}

pub fn unique_match<I, T>(mut iter: I, name: &str) -> Result<T, String>
where
    I: Iterator<Item = T>,
{
    match (iter.next(), iter.next()) {
        (None, _) => Err(format!("{} is not a known name", name)),
        (Some(_), Some(_)) => Err(format!("{} is not a unique name", name)),
        (Some(exec), None) => Ok(exec),
    }
}

pub mod add;
pub mod build;
pub mod check;
pub mod clean;
pub mod metadata;
pub mod opts;
pub mod run;
pub mod shared;
pub mod test;
