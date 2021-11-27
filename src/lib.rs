#![doc = include_str!("../README.md")]

extern crate self as bpaf;
use std::rc::Rc;
use std::str::FromStr;

pub mod params;
use crate::args::Word;
pub use crate::params::*;
mod args;
pub mod info;
pub use crate::args::Args;
use crate::info::Item;
pub use info::Info;

#[cfg(test)]
mod tests;

pub use info::{Meta, ParserInfo};

/// Compose several parsers to produce a struct with results
///
/// ```ignore
/// struct Res {
///     p1: bool,
///     p2: u32,
/// }
/// let p1: Parser<bool> = ...
/// let p2: Parser<u32> = ...
/// let p1p2: Parser<Res> = construct!(Res: p1, p2);
/// ```
#[macro_export]
macro_rules! construct {
    ($struct:ident : $( $field:ident ),* $(,)?) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($struct {$($field),*}, rest))
            }),
            meta: Meta::And(vec![ $($field.meta),*])
        }
    }
}

/// Compose several parsers to produce a tuple of results
///
/// ```ignore
/// let p1: Parser<bool> = ...
/// let p2: Parser<u32> = ...
/// let p1p2: Parser<(bool, u32)> = tuple!(p1, p2)
/// ```
#[macro_export]
macro_rules! tuple {
    ($($x:ident),* $(,)?) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($x, rest) = ($x.parse)(rest)?;)*
                Ok((($($x),*), rest))
            }),
            meta: Meta::And(vec![ $($x.meta),*])
        }
    }
}

/// Compose several parsers and call a function if parsers succeed
///
/// ```ignore
/// fn make_res(a: bool, b: u32) -> Res {...}
///
/// let p1: Parser<bool> = ...
/// let p2: Parser<u32> = ...
/// let p1p2: Parser<Res> = apply!(make_res: p1, p2);
/// ```
#[macro_export]
macro_rules! apply {
    ($fn:ident : $( $field:ident ),* $(,)?) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($fn($($field),*), rest))
            }),
            meta: Meta::And(vec![ $($field.meta),*])
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    /// Terminate and print this to stdout
    Stdout(String),
    /// Terminate and print this to stderr
    Stderr(String),
    /// Expected one of those values
    ///
    /// Used internally to generate better error messages
    Missing(Vec<Meta>),
}

impl Error {
    #[cfg(test)]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Error::Stderr(err) => err,
            Error::Stdout(_) | Error::Missing(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    #[cfg(test)]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Error::Stdout(err) => err,
            Error::Stderr(_) | Error::Missing(_) => {
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
    pub parse: Rc<dyn Fn(Args) -> Result<(T, Args), Error>>,
    pub meta: Meta,
}

impl<T> Parser<T> {
    pub fn pair<A, B>(a: Parser<A>, b: Parser<B>) -> Parser<(A, B)>
    where
        A: 'static + Clone,
        B: 'static + Clone,
    {
        tuple!(a, b)
    }

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
            meta: self.meta.clone().and(other.meta.clone()),
        }
    }

    // <|>
    pub fn or_else(self, other: Parser<T>) -> Parser<T>
    where
        T: 'static,
    {
        let parse = move |mut i: Args| -> Result<(T, Args), Error> {
            i.head = usize::MAX;
            // To generate less confusing error messages give priority to the left most flag/argument
            // from the command line:
            // So if program accepts only one of 3 flags: -a, -b and -c and all 3 are present
            // take the first one and reject the remaining ones.
            let (res, new_args) = match ((self.parse)(i.clone()), (other.parse)(i)) {
                (Ok((r1, a1)), Ok((r2, a2))) => {
                    if a1.head < a2.head {
                        Ok((r1, a1))
                    } else {
                        Ok((r2, a2))
                    }
                }
                (Ok(ok), Err(_)) => Ok(ok),
                (Err(_), Ok(ok)) => Ok(ok),
                (Err(e1), Err(e2)) => Err(e1.combine_with(e2)),
            }?;
            Ok((res, new_args))
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
                    panic!("many can't be used with non failing parser")
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
    pub fn guard<F>(self, m: F, message: &'static str) -> Parser<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let parse = move |i: Args| match (self.parse)(i) {
            Ok((ok, i)) if m(&ok) => Ok((ok, i)),
            Ok(_) => Err(Error::Stderr(message.to_string())), // TODO - see what exactly we tried to parse
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
        self.many().guard(|x| !x.is_empty(), "must not be empty")
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

    // apply failing transformation
    pub fn parse<F, B, E>(self, m: F) -> Parser<B>
    where
        F: Fn(T) -> Result<B, E> + 'static,
        T: 'static,
        E: ToString,
    {
        let parse = move |i: Args| {
            let (a, i) = (self.parse)(i)?;

            match m(a) {
                Ok(ok) => Ok((ok, i)),
                Err(e) => Err(Error::Stderr(match i.current {
                    Some(Word { utf8: Some(w), .. }) => {
                        format!("Couldn't parse {:?}: {}", w, e.to_string())
                    }
                    _ => format!("Couldn't parse: {}", e.to_string()),
                })),
            }
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
            e @ Err(Error::Stderr(_)) => e,
            Err(_) => Ok((val.clone(), i)),
        };
        Parser {
            parse: Rc::new(parse),
            meta: Meta::optional(self.meta),
        }
    }

    // use this default
    pub fn fallback_with<F, E>(self, val: F) -> Parser<T>
    where
        F: Fn() -> Result<T, E> + Clone + 'static,
        E: ToString,
        T: Clone + 'static,
    {
        let parse = move |i: Args| match (self.parse)(i.clone()) {
            Ok(ok) => Ok(ok),
            e @ Err(Error::Stderr(_)) => e,
            Err(_) => match val() {
                Ok(ok) => Ok((ok, i)),
                Err(e) => Err(Error::Stderr(e.to_string())),
            },
        };
        Parser {
            parse: Rc::new(parse),
            meta: Meta::optional(self.meta),
        }
    }

    // fallback to default
    pub fn default(self) -> Parser<T>
    where
        T: Default + 'static + Clone,
    {
        self.fallback(T::default())
    }

    pub fn help(self, msg: &'static str) -> Parser<T> {
        Parser {
            parse: self.parse,
            meta: Meta::decorate(self.meta, msg),
        }
    }
}

impl Parser<String> {
    pub fn from_str<T>(self) -> Parser<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        self.parse(|s| T::from_str(&s))
    }
}

pub fn run<T>(parser: ParserInfo<T>) -> T {
    let mut args: Vec<String> = std::env::args().collect();
    let prog_name = args.remove(0);
    let a = Args::from(args.as_slice());
    match run_inner(a, parser) {
        Ok(t) => t,
        Err(Error::Stdout(msg)) => {
            print!("{}", msg);
            std::process::exit(0);
        }
        Err(Error::Stderr(msg)) => {
            eprint!("{}", msg);
            std::process::exit(1);
        }
        Err(err) => unreachable!("failed: {:?}", err),
    }
}

fn run_inner<T>(args: Args, parser: ParserInfo<T>) -> Result<T, Error> {
    match (parser.parse)(args) {
        Ok((t, rest)) if rest.is_empty() => Ok(t),
        Ok((_, rest)) => Err(Error::Stderr(format!("unexpected {:?}", rest))),
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
        Err(err) => Err(err),
    }
}
