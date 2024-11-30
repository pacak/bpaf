#![allow(dead_code)]

mod visitor;

use named::Argument;
use named::Flag;
use named::Named;
use positional::Positional;

pub struct Cx<I>(I);

/// Make named item with short name
pub fn short(name: char) -> Cx<Named> {
    Cx(named::short(name))
}

pub fn long(name: &'static str) -> Cx<Named> {
    Cx(named::long(name))
}

pub fn positional<T>(meta: &'static str) -> Cx<Positional<T>> {
    Cx(positional::positional(meta))
}

/// # Named items
///
/// Parses something with name attached
impl Cx<Named> {
    pub fn short(mut self, name: char) -> Self {
        self.0.short(name);
        self
    }
    pub fn long(mut self, name: &'static str) -> Self {
        self.0.long(name);
        self
    }
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
    pub fn switch(self) -> Cx<Flag<bool>> {
        Cx(self.0.flag(true, Some(false)))
    }
    pub fn flag<T>(self, present: T, absent: T) -> Cx<Flag<T>> {
        Cx(self.0.flag(present, Some(absent)))
    }
    pub fn req_flag<T>(self, present: T) -> Cx<Flag<T>> {
        Cx(self.0.flag(present, None))
    }
    pub fn argument<T>(self, meta: &'static str) -> Cx<Argument<T>> {
        Cx(self.0.argument(meta))
    }
}

/// # Ready made parser
impl<T> Cx<Flag<T>> {
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
    // adjacent_to
}

impl<T> Cx<Argument<T>> {
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
}

/// # Positional items
///
/// Parses a positional item, usually with something attached
impl<T> Cx<Positional<T>> {
    pub fn help(mut self, help: String) -> Self {
        self.0.help(help);
        self
    }
}

mod named {
    use std::marker::PhantomData;

    pub struct Argument<T> {
        named: Named,
        ty: PhantomData<T>,
        meta: &'static str,
    }

    pub struct Flag<T> {
        named: Named,
        present: T,
        absent: Option<T>,
    }

    impl Named {
        pub(crate) fn flag<T>(self, present: T, absent: Option<T>) -> Flag<T> {
            Flag {
                named: self,
                present,
                absent,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Name<'a> {
        Short(char),
        Long(&'a str),
    }
    pub struct Named {
        names: Vec<Name<'static>>,
        help: Option<String>,
    }
    pub(crate) fn short(name: char) -> Named {
        Named {
            names: vec![Name::Short(name)],
            help: None,
        }
    }
    pub(crate) fn long(name: &'static str) -> Named {
        Named {
            names: vec![Name::Long(name)],
            help: None,
        }
    }
    impl Named {
        pub(crate) fn short(&mut self, name: char) {
            self.names.push(Name::Short(name));
        }
        pub(crate) fn long(&mut self, name: &'static str) {
            self.names.push(Name::Long(name));
        }
        pub(crate) fn help(&mut self, help: String) {
            self.help = Some(help);
        }
        pub(crate) fn argument<T>(self, meta: &'static str) -> Argument<T> {
            Argument {
                named: self,
                ty: PhantomData,
                meta,
            }
        }
    }
    impl<T> Flag<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.named.help = Some(help);
        }
    }

    impl<T> Argument<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.named.help = Some(help);
        }
    }
}

mod positional {
    use std::marker::PhantomData;
    pub struct Positional<T> {
        meta: &'static str,
        help: Option<String>,
        ty: PhantomData<T>,
    }
    pub(crate) fn positional<T>(meta: &'static str) -> Positional<T> {
        Positional {
            meta,
            help: None,
            ty: PhantomData,
        }
    }
    impl<T> Positional<T> {
        pub(crate) fn help(&mut self, help: String) {
            self.help = Some(help);
        }
    }
}

//pub mod ex2;
//pub mod ex3;
pub mod ex4;
