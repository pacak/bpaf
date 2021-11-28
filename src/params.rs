use std::ffi::OsString;

use super::*;
use crate::{
    args::Word,
    info::{ItemKind, Meta},
};

#[derive(Clone, Debug)]
pub struct Named {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
}

pub fn short(short: char) -> Named {
    Named {
        short: vec![short],
        long: Vec::new(),
        help: None,
    }
}

pub fn long(long: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: vec![long],
        help: None,
    }
}

impl Named {
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
        self
    }
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

impl Named {
    /// simple boolean flag
    pub fn switch(self) -> Flag<bool> {
        Flag {
            present: true,
            absent: Some(false),
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }
    pub fn req_switch(self) -> Flag<bool> {
        Flag {
            present: true,
            absent: None,
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    /// present/absent value flag
    pub fn flag<T>(self, present: T, absent: T) -> Flag<T> {
        Flag {
            present,
            absent: Some(absent),
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    /// required flag
    pub fn req_flag<T>(self, present: T) -> Flag<T> {
        Flag {
            present,
            absent: None,
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    pub fn argument(self) -> Argument {
        Argument {
            short: self.short,
            long: self.long,
            help: self.help,
            metavar: Some("ARG"),
        }
    }
}

pub fn command<T, M>(name: &'static str, help: M, p: ParserInfo<T>) -> Parser<T>
where
    T: 'static,
    M: Into<String>,
{
    let parse = move |mut i: Args| match i.take_word(name) {
        Some(i) => (p.parse)(i),
        None => Err(Error::Stderr(format!("expected {}", name))),
    };
    let meta = Meta::from(Item {
        short: None,
        long: Some(name),
        metavar: None,
        help: Some(help.into()),
        kind: ItemKind::Command,
    });
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

#[derive(Default)]
pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
}

impl<T> Flag<T> {
    pub fn build(self) -> Parser<T>
    where
        T: Clone + 'static,
    {
        let item = Item {
            short: self.short.first().copied(),
            long: self.long.first().copied(),
            metavar: None,
            help: self.help,
            kind: ItemKind::Flag,
        };
        let required = self.absent.is_none();
        let meta = item.required(required);

        let missing = if required {
            Error::Missing(vec![meta.clone()])
        } else {
            Error::Stdout(String::new())
        };

        let parse = move |mut i: Args| {
            for &short in self.short.iter() {
                if let Some(i) = i.take_short_flag(short) {
                    return Ok((self.present.clone(), i));
                }
            }
            for long in self.long.iter() {
                if let Some(i) = i.take_long_flag(long) {
                    return Ok((self.present.clone(), i));
                }
            }
            Ok((
                self.absent.as_ref().ok_or_else(|| missing.clone())?.clone(),
                i,
            ))
        };
        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }
}

impl<T> Flag<T> {
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

pub struct Argument {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
    metavar: Option<&'static str>,
}

impl Argument {
    fn build_both(self) -> Parser<Word> {
        let item = Item {
            kind: ItemKind::Flag,
            short: self.short.first().copied(),
            long: self.long.first().copied(),
            metavar: self.metavar,
            help: self.help,
        };
        let meta = item.required(true);
        let meta2 = meta.clone();
        let parse = move |mut i: Args| {
            for &short in self.short.iter() {
                if let Some((w, c)) = i.take_short_arg(short)? {
                    return Ok((w, c));
                }
            }
            for long in self.long.iter() {
                if let Some((w, c)) = i.take_long_arg(long)? {
                    return Ok((w, c));
                }
            }
            Err(Error::Missing(vec![meta2.clone()]))
        };

        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }

    pub fn build(self) -> Parser<String> {
        self.build_both().parse(|x| x.utf8.ok_or("not utf8")) // TODO
    }

    pub fn build_os(self) -> Parser<OsString> {
        self.build_both().map(|x| x.os)
    }

    pub fn metavar(mut self, metavar: &'static str) -> Self {
        self.metavar = Some(metavar);
        self
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}
