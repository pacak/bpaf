#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![doc = include_str!("../README.md")]

use std::rc::Rc;
use std::str::FromStr;

pub mod params;

mod args;
pub mod info;

use crate::{args::Word, info::Error, info::Item};

#[cfg(test)]
mod tests;
#[doc(inline)]
pub use crate::args::Args;
#[doc(inline)]
pub use crate::info::{Info, Meta, ParserInfo};
#[doc(inline)]
pub use crate::params::*;

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

#[doc(hidden)]
/// A bit more user friendly alias for parsing function
pub type DynParse<T> = dyn Fn(Args) -> Result<(T, Args), Error>;

/// Simple or composed argument parser
#[derive(Clone)]
pub struct Parser<T> {
    #[doc(hidden)]
    /// Parsing function
    pub parse: Rc<DynParse<T>>,
    #[doc(hidden)]
    /// Included information about the parser
    pub meta: Meta,
}

impl<T> Parser<T> {
    /// succeed without consuming anything
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

    #[doc(hidden)]
    /// <*>
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

    /// If first parser fails - try the second one
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

    /// Fail with a fixed error message
    pub fn fail<M>(msg: M) -> Parser<T>
    where
        String: From<M>,
        M: Clone + 'static,
    {
        Parser {
            meta: Meta::Empty,
            parse: Rc::new(move |_| Err(Error::Stderr(String::from(msg.clone())))),
        }
    }

    /// zero or more
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

    /// Validate or fail with a message
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

    /// one or more
    pub fn some(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        self.many().guard(|x| !x.is_empty(), "must not be empty")
    }

    /// zero or one
    pub fn optional(self) -> Parser<Option<T>>
    where
        T: 'static + Clone,
    {
        self.map(Some).or_else(Parser::pure(None))
    }

    /// apply pure transformation
    pub fn map<F, B>(self, map: F) -> Parser<B>
    where
        F: Fn(T) -> B + 'static,
        T: 'static,
    {
        let parse = move |args: Args| {
            let (t, args) = (self.parse)(args)?;
            Ok((map(t), args))
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    // apply failing transformation
    pub fn parse<F, B, E>(self, map: F) -> Parser<B>
    where
        F: Fn(T) -> Result<B, E> + 'static,
        T: 'static,
        E: ToString,
    {
        let parse = move |args: Args| {
            let (t, args) = (self.parse)(args)?;

            match map(t) {
                Ok(ok) => Ok((ok, args)),
                Err(e) => Err(Error::Stderr(match args.current {
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

impl<T> ParserInfo<T> {
    /// Execute the [ParserInfo], extract a parsed value or print some diagnostic and exit
    ///
    /// ```no_run
    /// use bpaf::*;
    ///
    /// let verbose = short('v').req_flag(()).many().map(|xs|xs.len());
    /// let info = Info::default().descr("Takes verbosity flag and does nothing else");
    ///
    /// let opt = info.for_parser(verbose).run();
    /// // At this point `opt` contains number of times `-v` was specified on a command line
    /// # drop(opt)
    /// ```
    pub fn run(self) -> T {
        let mut args = Args::default();
        let mut pos_only = false;
        for arg in std::env::args_os() {
            args.push(arg, &mut pos_only);
        }

        match self.run_inner(args) {
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

    fn run_inner(self, args: Args) -> Result<T, Error> {
        match (self.parse)(args) {
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
}
