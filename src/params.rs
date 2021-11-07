use super::*;
use crate::info::{ItemKind, Meta};

pub struct Named {
    short: Option<char>,
    long: Option<&'static str>,
    help: Option<&'static str>,
}

pub fn short(short: char) -> Named {
    Named {
        short: Some(short),
        long: None,
        help: None,
    }
}

pub fn long(long: &'static str) -> Named {
    Named {
        short: None,
        long: Some(long),
        help: None,
    }
}

impl Named {
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
            metavar: None,
        }
    }
}

pub fn command<T>(name: &'static str, help: &'static str, p: ParserInfo<T>) -> Parser<T>
where
    T: 'static,
{
    let parse = move |mut i: Args| match i.take_word(name) {
        Some(i) => (p.parse)(i),
        None => Err(Error::Stderr(format!("expected {}", name))),
    };
    let meta = Meta::from(Item {
        short: None,
        long: Some(name),
        metavar: None,
        help: Some(help),
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
    short: Option<char>,
    long: Option<&'static str>,
    help: Option<&'static str>,
}

impl<T> Flag<T> {
    pub fn build(self) -> Parser<T>
    where
        T: Clone + 'static,
    {
        let item = Item {
            short: self.short,
            long: self.long,
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
            if let Some(i) = self.short.and_then(|f| i.take_short_flag(f)) {
                return Ok((self.present.clone(), i));
            }
            if let Some(i) = self.long.and_then(|f| i.take_long_flag(f)) {
                return Ok((self.present.clone(), i));
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
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}

pub struct Argument {
    short: Option<char>,
    long: Option<&'static str>,
    help: Option<&'static str>,
    metavar: Option<&'static str>,
}

impl Argument {
    pub fn build(self) -> Parser<String> {
        let item = Item {
            kind: ItemKind::Flag,
            short: self.short,
            long: self.long,
            metavar: self.metavar,
            help: self.help,
        };
        let meta = item.required(true);
        let meta2 = meta.clone();
        let parse = move |mut i: Args| {
            if let Some(v) = self.short.and_then(|f| i.take_short_arg(f)) {
                return Ok(v);
            }
            if let Some(v) = self.long.and_then(|f| i.take_long_arg(f)) {
                return Ok(v);
            }
            Err(Error::Missing(vec![meta2.clone()]))
        };

        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }
    pub fn metavar(mut self, metavar: &'static str) -> Self {
        self.metavar = Some(metavar);
        self
    }
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }
}
