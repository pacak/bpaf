use std::{marker::PhantomData, str::FromStr};

use crate::{
    long,
    parsers::{NamedArg, ParseArgument, ParseFlag, ParsePositional},
    positional, short, Parser,
};

pub struct SimpleParser<T, S> {
    state: PhantomData<S>,
    x: SimpleEnum<T>,
}

pub enum Named {}
pub enum Positional {}

enum SimpleEnum<T> {
    Named(NamedArg),
    Positional(ParsePositional<T>),
    // Any(ParseAny<A, B, C>)
}

impl<T, S> SimpleParser<T, S> {
    pub fn help(self) {
        match self.x {
            SimpleEnum::Named(x) => todo!(),
            SimpleEnum::Positional(x) => todo!(),
        }
    }
}

impl<T> SimpleParser<T, Positional> {
    pub fn positional(meta: &'static str) -> Self {
        SimpleParser {
            x: SimpleEnum::Positional(positional(meta)),
            state: PhantomData,
        }
    }
}

impl<T> SimpleParser<T, Named> {
    /// See [`short`]
    pub fn with_short(name: char) -> Self {
        let x = SimpleEnum::Named(short(name));
        SimpleParser {
            state: PhantomData,
            x,
        }
    }
    /// See [`long`]
    pub fn with_long(name: &'static str) -> Self {
        let x = SimpleEnum::Named(long(name));
        SimpleParser {
            state: PhantomData,
            x,
        }
    }

    pub fn short(self, name: char) -> Self {
        match self.x {
            SimpleEnum::Named(n) => SimpleParser {
                state: PhantomData,
                x: SimpleEnum::Named(n.short(name)),
            },
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }

    pub fn long(self, name: &'static str) -> Self {
        match self.x {
            SimpleEnum::Named(n) => SimpleParser {
                state: PhantomData,
                x: SimpleEnum::Named(n.long(name)),
            },
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }
}

impl<T> SimpleParser<T, Named>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    pub fn argument(self, meta: &'static str) -> ParseArgument<T> {
        match self.x {
            SimpleEnum::Named(n) => n.argument(meta),
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }
}

impl<T> SimpleParser<T, Named>
where
    T: Clone + 'static,
{
    pub fn flag(self, present: T, absent: T) -> ParseFlag<T> {
        match self.x {
            SimpleEnum::Named(n) => n.flag(present, absent),
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }
    pub fn req_flag(self, present: T) -> ParseFlag<T> {
        match self.x {
            SimpleEnum::Named(n) => n.req_flag(present),
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }
}

impl SimpleParser<bool, Named> {
    pub fn switch(self) -> ParseFlag<bool> {
        match self.x {
            SimpleEnum::Named(n) => n.switch(),
            SimpleEnum::Positional(_) => unreachable!(),
        }
    }
}

impl<T> Parser<T> for SimpleParser<T, Positional>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    fn eval(&self, args: &mut crate::State) -> Result<T, crate::Error> {
        match &self.x {
            SimpleEnum::Named(_) => unreachable!(),
            SimpleEnum::Positional(p) => p.eval(args),
        }
    }

    fn meta(&self) -> crate::Meta {
        match &self.x {
            SimpleEnum::Named(_) => unreachable!(),
            SimpleEnum::Positional(p) => p.meta(),
        }
    }
}
