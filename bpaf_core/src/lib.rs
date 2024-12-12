mod visitor;

pub mod executor;
pub mod parsers;
use executor::Fragment;
use named::{Argument, Flag, Named};
use positional::Positional;
mod error;

/// Parser with extra typestate information
///
/// **bpaf** exposes all of it's functionality using
/// [Fluent interface](https://en.wikipedia.org/wiki/Fluent_interface?useskin=vector), this
/// datatype collects all possible intermediate representations along with all the
/// methods they support in the same place.
///
/// In additon to inherent methods, most of the `Cx` states implement [`Parser`] trait with more
/// operations.
///
/// While this documentation explains *fluent interface* specifically, most of it applies
// TODO - add link
/// to the derive API as well, the main difference is that instead of methods being chained
/// on the parser - you list them inside derive annotations. In both of those examples `many` refers to
/// [`Parser::many`].
///
/// Fluent API:
/// ```ignore
/// ...
/// let field = some_parser().many();
/// ...
/// ```
///
/// Derive API:
/// ```ignore
/// ...
/// #[bpaf(external(some_parser), many)]
/// field: Vec<String>
/// ...
/// ```
///
///
/// Some of the notable states
///
/// - [Named items](#named-items)
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

/// # Parser for a named item
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

    use crate::{
        error::{Error, MissingItem},
        executor::{Ctx, Fragment, NamedFut},
        Parser,
    };

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

    impl Parser<Name<'static>> for Named {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> Fragment<'a, Name<'static>> {
            Box::pin(NamedFut {
                name: self.names.as_slice(),
                ctx,
                task_id: None,
                meta: None, // TODO - use separate instances for Argument and Flag
            })
        }
    }

    impl<T> Parser<T> for Flag<T>
    where
        T: std::fmt::Debug + Clone + 'static,
    {
        fn run<'a>(&'a self, input: Ctx<'a>) -> Fragment<'a, T> {
            Box::pin(async move {
                match self.named.run(input).await {
                    Ok(_) => Ok(self.present.clone()),
                    Err(e) if e.handle_with_fallback() => match self.absent.as_ref().cloned() {
                        Some(v) => Ok(v),
                        None => Err(e),
                    },
                    Err(e) => Err(e),
                }
            })
        }
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
        Short(char, [u8; 4]),
        Long(&'a str),
    }

    impl Name<'_> {
        pub(crate) fn as_bytes(&self) -> &[u8] {
            match self {
                Name::Short(_, a) => a.as_slice(),
                Name::Long(l) => l.as_bytes(),
            }
        }

        pub(crate) fn short(name: char) -> Name<'static> {
            let mut buf = [0; 4];
            name.encode_utf8(&mut buf);

            Name::Short(name, buf)
        }
    }

    impl std::borrow::Borrow<[u8]> for Name<'_> {
        fn borrow(&self) -> &[u8] {
            self.as_bytes()
        }
    }

    pub struct Named {
        names: Vec<Name<'static>>,
        help: Option<String>,
    }

    pub(crate) fn short(name: char) -> Named {
        let mut buf = [0; 4];
        name.encode_utf8(&mut buf);
        Named {
            names: vec![Name::Short(name, buf)],
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
            let mut buf = [0; 4];
            name.encode_utf8(&mut buf);
            self.names.push(Name::Short(name, buf));
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
    use std::{marker::PhantomData, str::FromStr};

    use crate::{
        error::{Error, Message},
        executor::{Ctx, PositionalFut},
        Parser,
    };
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

    impl<T> Parser<T> for Positional<T>
    where
        T: std::fmt::Debug + 'static + FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        fn run<'a>(&'a self, ctx: Ctx<'a>) -> crate::executor::Fragment<'a, T> {
            Box::pin(async {
                let s = PositionalFut {
                    ctx,
                    task_id: None,
                    meta: self.meta,
                }
                .await;
                T::from_str(s?).map_err(|e| Error {
                    message: Message::ParseFailed(None, format!("{e}")),
                })
            })
        }
    }
}

pub use executor::Parser;

impl<P, T> Parser<T> for Cx<P>
where
    P: Parser<T>,
    T: 'static + std::fmt::Debug,
{
    fn run<'a>(&'a self, ctx: executor::Ctx<'a>) -> Fragment<'a, T> {
        self.0.run(ctx)
    }
}
