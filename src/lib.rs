use std::rc::Rc;

pub mod params;
pub use crate::params::*;
mod args;
pub mod info;
pub use crate::args::Args;
use crate::info::Item;

#[cfg(test)]
mod tests;

use info::{Meta, ParserInfo};

#[macro_export]
macro_rules! curry {
    (|$($pat:ident),*| $expr:expr) => ($(move |$pat| )* $expr);
}

#[derive(Clone, Debug)]
pub(crate) enum Error {
    Stdout(String),
    Stderr(String),
    Missing(Vec<Meta>),
    Unexpected(String),
}

impl Error {
    #[cfg(test)]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Error::Stderr(err) => err,
            Error::Stdout(_) | Error::Missing(_) | Error::Unexpected(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    #[cfg(test)]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Error::Stdout(err) => err,
            Error::Stderr(_) | Error::Missing(_) | Error::Unexpected(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }

    pub fn combine_with(self, other: Self) -> Self {
        match (self, other) {
            // finalized error takes priority
            (a @ Error::Stderr(_), _) => a,
            (_, b @ Error::Stderr(_)) => b,

            // missing elements are combined
            (Error::Missing(mut a), Error::Missing(mut b)) => {
                a.append(&mut b);
                Error::Missing(a)
            }
            // missing takes priority
            (a @ Error::Missing(_), _) => a,
            (_, b @ Error::Missing(_)) => b,

            // first error wins,
            (a, _) => a,
        }
    }
}

#[derive(Clone)]
pub struct Parser<T> {
    parse: Rc<dyn Fn(Args) -> Result<(T, Args), Error>>,
    meta: Meta,
}
/*
#[derive(Default)]
pub struct Argument {
    short: Option<char>,
    long: Option<&'static str>,
    help: Option<&'static str>,
    metavar: Option<&'static str>,
}

impl Argument {
    pub fn build(self) -> Parser<String> {
        let parse = move |mut i: Args| {
            if let Some(v) = self.short.and_then(|f| i.take_short_arg(f)) {
                return Ok(v);
            }
            if let Some(v) = self.long.and_then(|f| i.take_long_arg(f)) {
                return Ok(v);
            }
            Err(format!("missing {:?}", (self.short, self.long)))
        };
        let meta = Vec::new();
        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }

    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }
    pub fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
    pub fn metavar(mut self, metavar: &'static str) -> Self {
        self.metavar = Some(metavar);
        self
    }
}*/

pub struct Unnamed;
pub struct Named;

// TODO:
// - switch - boolean flag
// - flag - enabled/disabled flag
// - flag' - enabled only flag
// - option - argument parsed from string
// - argument - positional argument
// - subcommand

impl<T> Parser<T> {
    // succeed without consuming anything
    pub fn pure(val: T) -> Parser<T>
    where
        T: 'static + Clone,
    {
        let parse = move |i| Ok((val.clone(), i));
        Parser {
            parse: Rc::new(parse),
            meta: Meta::Id,
        }
    }

    // <*>
    pub fn ap<A, B>(self, other: Parser<A>) -> Parser<B>
    where
        T: Fn(A) -> B + 'static,
        A: 'static,
    {
        let parse = move |i| {
            let (t, rest) = (self.parse)(i)?;
            let (a, rest) = (other.parse)(rest)?;
            Ok((t(a), rest))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta.and(other.meta),
        }
    }

    // <|>
    pub fn or_else(self, other: Parser<T>) -> Parser<T>
    where
        T: 'static,
    {
        let parse = move |i: Args| match (self.parse)(i.clone()) {
            Ok(ok) => Ok(ok),
            Err(err1) => match (other.parse)(i) {
                Ok(ok) => Ok(ok),
                Err(err2) => Err(err1.combine_with(err2)),
            },
        };

        Parser {
            parse: Rc::new(parse),
            meta: self.meta.or(other.meta),
        }
    }

    // always fails
    pub fn empty() -> Parser<T> {
        Parser {
            meta: Meta::Empty,
            parse: Rc::new(|_| Err(Error::Stderr(String::from("empty")))),
        }
    }

    // zero or more
    pub fn many(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        let parse = move |mut i: Args| {
            let mut res = Vec::new();
            let mut size = i.len();
            while let Ok((elt, new_i)) = (self.parse)(i.clone()) {
                let new_size = new_i.len();
                if new_size < size {
                    size = new_size
                } else {
                    panic!()
                }
                i = new_i;
                res.push(elt);
            }
            Ok((res, i))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta.many(),
        }
    }

    //
    pub fn guard<F>(self, m: F) -> Parser<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let parse = move |i: Args| match (self.parse)(i) {
            Ok((ok, i)) if m(&ok) => Ok((ok, i)),
            Ok(_) => Err(Error::Stderr(format!("Guard failed"))), // TODO - see what exactly we tried to parse
            Err(err) => Err(err),
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    // one or more
    pub fn some(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        self.many().guard(|x| !x.is_empty())
    }

    // zero or one
    pub fn optional(self) -> Parser<Option<T>>
    where
        T: 'static + Clone,
    {
        self.map(Some).or_else(Parser::pure(None))
    }

    // apply pure transformation
    pub fn map<F, B>(self, m: F) -> Parser<B>
    where
        F: Fn(T) -> B + 'static,
        T: 'static,
    {
        let parse = move |i: Args| {
            let (a, b) = (self.parse)(i)?;
            Ok((m(a), b))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    // apply failing transformation transformation
    pub fn parse<F, B, E>(self, m: F) -> Parser<B>
    where
        F: Fn(T) -> Result<B, E> + 'static,
        T: 'static,
        E: ToString,
    {
        let parse = move |i: Args| {
            let (a, i) = (self.parse)(i)?;
            Ok((m(a).map_err(|e| Error::Stderr(e.to_string()))?, i))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    // use this default
    pub fn fallback(self, val: T) -> Parser<T>
    where
        T: Clone + 'static,
    {
        let parse = move |i: Args| match (self.parse)(i.clone()) {
            Ok(ok) => Ok(ok),
            Err(_) => Ok((val.clone(), i)),
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    // fallback to default
    pub fn default(self) -> Parser<T>
    where
        T: Default + 'static + Clone,
    {
        self.fallback(T::default())
    }
}

pub fn run<T>(parser: ParserInfo<T>) -> T {
    let mut args: Vec<String> = std::env::args().collect();
    let prog_name = args.remove(0);
    let a = Args::from(args.as_slice());
    match run_inner(a, parser) /*(parser.parse)(a) */{
        Ok(t) => t,
        Err(Error::Stdout(msg)) => {
            print!("{}", msg);
            std::process::exit(0);
        }
        Err(Error::Stderr(msg)) => {
            eprint!("{}", msg);
            std::process::exit(1);
        }
        Err(err) => panic!("failed: {:?}", err),
    }
}

fn run_inner<T>(args: Args, parser: ParserInfo<T>) -> Result<T, Error> {
    match (parser.parse)(args) {
        Ok((t, rest)) if rest.is_empty() => Ok(t),
        Ok((_, rest)) => Err(Error::Unexpected(format!("unexpected {:?}", rest))),
        Err(Error::Missing(metas)) => {
            if metas.len() == 1 {
                Err(Error::Stderr(format!("Expected {}", metas[0])))
            } else {
                use std::fmt::Write;
                let mut res = String::new();
                write!(res, "Expected one of ").unwrap();
                for (ix, x) in metas.iter().enumerate() {
                    write!(res, "{}", x).unwrap();
                    if metas.len() > ix + 1 {
                        write!(res, ", ").unwrap();
                    }
                }
                Err(Error::Stderr(res))
            }
        }
        Err(err) => Err(err), // TODO
    }
}
