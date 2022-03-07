#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![doc = include_str!("../README.md")]

use std::rc::Rc;
use std::str::FromStr;

pub mod params;

mod args;

#[doc(hidden)]
pub mod info;

use crate::{args::Word, info::Error, info::Item};

#[cfg(test)]
mod tests;
#[doc(inline)]
pub use crate::args::Args;
#[doc(inline)]
pub use crate::info::{Info, Meta, OptionParser};
#[doc(inline)]
pub use crate::params::*;

/// Compose several parsers to produce a single result
///
/// Every parser must succeed in order to produce a result
///
/// Each parser must be present in a local scope and
/// have the same name as struct field.
///
/// ```rust
/// # use bpaf::*;
/// struct Res {
///     a: bool,
///     b: u32,
/// }
/// let a: Parser<bool> = short('a').switch();
/// let b: Parser<u32> = short('b').argument("B").from_str();
/// let res: Parser<Res> = construct!(Res { a, b });
/// # drop(res);
/// ```
#[macro_export]
macro_rules! construct {
    ($struct:ident { $( $field:ident ),* $(,)? }) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($struct {$($field),*}, rest))
            }),
            meta: $crate::Meta::And(vec![ $($field.meta),*])
        }
    };
    ($enum:ident :: $constr:ident { $( $field:ident ),* $(,)? }) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($enum :: $constr{$($field),*}, rest))
            }),
            meta: $crate::Meta::And(vec![ $($field.meta),*])
        }
    };
    ($struct:ident ( $( $field:ident ),* $(,)? )) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($struct ($($field),*), rest))
            }),
            meta: $crate::Meta::And(vec![ $($field.meta),*])
        }
    };
    ($enum:ident :: $constr:ident ( $( $field:ident ),* $(,)? )) => {
        Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($field, rest) = ($field.parse)(rest)?;)*
                Ok(($enum :: $constr($($field),*), rest))
            }),
            meta: $crate::Meta::And(vec![ $($field.meta),*])
        }
    };
    ($($x:ident),* $(,)?) => {
        $crate::Parser {
            parse: ::std::rc::Rc::new(move |rest| {
                $(let ($x, rest) = ($x.parse)(rest)?;)*
                Ok((($($x),*), rest))
            }),
            meta: $crate::Meta::And(vec![ $($x.meta),*])
        }
    };
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
    /// Wrap a value into a `Parser`
    ///
    /// Parser will produce `T` without consuming anything from the command line, can be useful
    /// with [`construct!`].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let a = long("flag-a").switch();
    /// let b = Parser::pure(42u32);
    /// let t: Parser<(bool, u32)> = construct!(a, b);
    /// # drop(t)
    /// ```
    #[must_use]
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
    #[must_use]
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
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let a = short('a').switch();
    /// let b = short('b').switch();
    ///
    /// // Parser will accept either `-a` or `-b` on a command line but not both at once.
    /// let a_or_b: Parser<bool> = a.or_else(b);
    /// # drop(a_or_b);
    /// ```
    ///
    /// # Performance
    ///
    /// If first parser succeeds - second one will be called anyway to produce a
    /// better error message for combinations of mutually exclusive parsers:
    ///
    /// Suppose program accepts one of two mutually exclusive switches `-a` and `-b`
    /// and both are present error message should point at the second flag
    ///
    #[must_use]
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
                (Ok(ok), Err(_)) | (Err(_), Ok(ok)) => Ok(ok),
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
    /// ```rust
    /// # use bpaf::*;
    /// let a = short('a').switch();
    /// let no_a = Parser::fail("Custom error message for missing -a");
    ///
    /// // Parser will produce a custom error message if `-a` is not specified
    /// let a_: Parser<bool> = a.or_else(no_a);
    /// # drop(a_);
    /// ```
    #[must_use]
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

    /// Consume zero or more items from a command line
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector which can be empty if no `-n` specified
    /// let n: Parser<Vec<u32>> = short('n').argument("NUM").from_str::<u32>().many();
    /// # drop(n);
    /// ```
    ///
    /// # Panics
    /// Panics if parser succeeds without consuming any input: any parser modified with
    /// `many` must consume something,
    #[must_use]
    pub fn many(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        let parse = move |mut i: Args| {
            let mut res = Vec::new();
            let mut size = i.len();
            while let Ok((elt, new_i)) = (self.parse)(i.clone()) {
                let new_size = new_i.len();
                #[allow(clippy::panic)]
                if new_size < size {
                    size = new_size;
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
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// // Parser will reject values greater than 10
    /// let n = n.guard(|v| *v <= 10, "Values greater than 10 are only available in the DLC pack!");
    /// # drop(n);
    /// ```
    #[must_use]
    pub fn guard<F>(self, m: F, message: &'static str) -> Parser<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let parse = move |i: Args| match (self.parse)(i) {
            Ok((ok, i)) if m(&ok) => Ok((ok, i)),
            Ok(_) => Err(Error::Stderr(message.to_owned())), // TODO - see what exactly we tried to parse
            Err(err) => Err(err),
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    /// Consume one or more items from a command line
    ///
    /// ```rust
    /// # use bpaf::*;
    /// // parser will accept multiple `-n` arguments:
    /// // `-n 1, -n 2, -n 3`
    /// // and return all of them as a vector. At least one `-n` argument is required.
    /// let n: Parser<Vec<u32>> = short('n').argument("NUM").from_str::<u32>().some();
    /// # drop(n);
    /// ```
    #[must_use]
    pub fn some(self) -> Parser<Vec<T>>
    where
        T: 'static,
    {
        self.many().guard(|x| !x.is_empty(), "must not be empty")
    }

    /// Turn a required parser into optional
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n: Parser<u32> = short('n').argument("NUM").from_str();
    /// // if `-n` is not specified - parser will return `None`
    /// let n: Parser<Option<u32>> = n.optional();
    /// # drop(n);
    /// ```
    pub fn optional(self) -> Parser<Option<T>>
    where
        T: 'static + Clone,
    {
        self.map(Some).fallback(None)
    }

    /// Apply a pure transformation to a contained value
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n: Parser<u32> = short('n').argument("NUM").from_str();
    /// // produced value is now twice as large
    /// let n = n.map(|v| v * 2);
    /// # drop(n);
    /// ```
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

    /// Apply a failing transformation
    ///
    /// See also [`from_str`][Parser::from_str]
    /// ```rust
    /// # use bpaf::*;
    /// let s: Parser<String> = short('n').argument("NUM");
    /// // Try to parse String into u32 or fail during the parsing
    /// use std::str::FromStr;
    /// let n = s.map(|s| u32::from_str(&s));
    /// # drop(n);
    /// ```
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
                Err(e) => Err(Error::Stderr(
                    if let Some(Word { utf8: Some(w), .. }) = args.current {
                        format!("Couldn't parse {:?}: {}", w, e.to_string())
                    } else {
                        format!("Couldn't parse: {}", e.to_string())
                    },
                )),
            }
        };
        Parser {
            parse: Rc::new(parse),
            meta: self.meta,
        }
    }

    /// Use this value as default if value is not present on a command line
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().fallback(42);
    /// # drop(n)
    /// ```
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

    /// Use value produced by this function as default if value is not present
    ///
    /// Would still fail if value is present but failure comes from some transformation
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>();
    /// let n = n.fallback_with(|| Result::<u32, String>::Ok(42));
    /// # drop(n)
    /// ```
    #[must_use]
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

    /// Parse `T` or fallback to `T::default()`
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let n = short('n').argument("NUM").from_str::<u32>().default();
    /// # drop(n)
    /// ```
    #[must_use]
    pub fn default(self) -> Parser<T>
    where
        T: Default + 'static + Clone,
    {
        self.fallback(T::default())
    }

    /// Attach help message to a complex parser
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let width = short('w').argument("PX").from_str::<u32>();
    /// let height = short('h').argument("PX").from_str::<u32>();
    /// let rect = construct!(width, height).help("take a rectangle");
    /// # drop(rect);
    /// ```
    /// See `examples/rectangle.rs` for a complete example
    #[must_use]
    pub fn help(self, msg: &'static str) -> Parser<T> {
        Self {
            parse: self.parse,
            meta: Meta::decorate(self.meta, msg),
        }
    }
}

impl Parser<String> {
    /// Parse stored [`String`] using [`FromStr`] instance
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let speed = short('s').argument("SPEED").from_str::<f64>();
    /// // at this point program would accept things like "-s 3.1415"
    /// // but reject "-s pi"
    /// # drop(speed)
    /// ```
    #[must_use]
    pub fn from_str<T>(self) -> Parser<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        self.parse(|s| T::from_str(&s))
    }
}

impl<T> OptionParser<T> {
    /// Execute the [`OptionParser`], extract a parsed value or print some diagnostic and exit
    ///
    /// ```no_run
    /// # use bpaf::*;
    /// let verbose = short('v').req_flag(()).many().map(|xs|xs.len());
    /// let info = Info::default().descr("Takes verbosity flag and does nothing else");
    ///
    /// let opt = info.for_parser(verbose).run();
    /// // At this point `opt` contains number of repetitions of `-v` on a command line
    /// # drop(opt)
    /// ```
    #[must_use]
    pub fn run(self) -> T {
        let mut args = Args::default();
        let mut pos_only = false;
        for arg in std::env::args_os().skip(1) {
            args.push(arg, &mut pos_only);
        }

        match self.run_inner(args) {
            Ok(t) => t,
            Err(Error::Stdout(msg)) => {
                println!("{}", msg);
                std::process::exit(0);
            }
            Err(Error::Stderr(msg)) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
            #[allow(clippy::unreachable)]
            Err(err) => unreachable!("failed: {:?}", err),
        }
    }

    fn run_inner(self, args: Args) -> Result<T, Error> {
        match (self.parse)(args) {
            Ok((t, rest)) if rest.is_empty() => Ok(t),
            Ok((_, rest)) => Err(Error::Stderr(format!("unexpected {:?}", rest))),
            Err(Error::Missing(metas)) => Err(Error::Stderr(format!(
                "Expected {}, pass --help for usage information",
                Meta::Or(metas)
            ))),
            Err(err) => Err(err),
        }
    }
}
