use std::rc::Rc;

use crate::{args::Args, params::short, Error, Parser};

#[derive(Clone, Debug)]
pub struct Item {
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub metavar: Option<&'static str>,
    pub help: Option<&'static str>,
    pub is_command: bool,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_command {
            return write!(f, "COMMAND");
        }
        match (self.short, self.long) {
            (None, None) => unreachable!(),
            (None, Some(l)) => write!(f, "--{}", l),
            (Some(s), None) => write!(f, "-{}", s),
            (Some(s), Some(l)) => write!(f, "-{}|--{}", s, l),
        }
    }
}

impl Item {
    pub fn required(self, required: bool) -> Meta {
        if required {
            Meta::Required(Box::new(Meta::Item(self)))
        } else {
            Meta::Optional(Box::new(Meta::Item(self)))
        }
    }

    pub fn name_len(&self) -> usize {
        let mut res = 0;
        res += match self.long {
            Some(s) => s.len() + 3,
            None => 0,
        };
        res += match self.metavar {
            Some(s) => s.len() + 2,
            None => 0,
        };
        res
    }
}

#[derive(Clone, Debug)]
pub enum Meta {
    Empty,
    And(Box<Meta>, Box<Meta>),
    Or(Vec<Meta>),
    Required(Box<Meta>),
    Optional(Box<Meta>),
    Item(Item),
    Many(Box<Meta>),
    Id,
}

impl Meta {
    pub fn is_required(&self) -> bool {
        match self {
            Meta::Empty => false,
            Meta::And(a, b) => a.is_required() || b.is_required(),
            Meta::Or(xs) => xs.iter().all(|x| x.is_required()),
            Meta::Required(_) => true,
            Meta::Optional(_) => false,
            Meta::Item(_) => todo!(),
            Meta::Many(_) => false,
            Meta::Id => todo!(),
        }
    }
    pub fn or(self, other: Meta) -> Self {
        match (self, other) {
            (Meta::Id, b) => b,
            (Meta::Empty, b) => b,
            (a, Meta::Empty) => a,
            (a, Meta::Id) => a,
            (Meta::Or(mut a), Meta::Or(mut b)) => {
                a.append(&mut b);
                Meta::Or(a)
            }
            (Meta::Or(mut a), b) => {
                a.push(b);
                Meta::Or(a)
            }
            (a, Meta::Or(mut b)) => {
                b.push(a);
                Meta::Or(b)
            }
            (a, b) => Meta::Or(vec![a, b]),
        }
    }
    pub fn and(self, other: Meta) -> Self {
        match (self, other) {
            (Meta::Id, a) => a,
            (b, Meta::Id) => b,
            (a, b) => Meta::And(Box::new(a), Box::new(b)),
        }
    }
    pub fn optional(self) -> Self {
        Meta::Optional(Box::new(self))
    }
    pub fn required(self) -> Self {
        Meta::Required(Box::new(self))
    }
    pub fn many(self) -> Self {
        Meta::Many(Box::new(self))
    }

    pub fn commands(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res);
        res.retain(|i| i.is_command);
        res
    }

    pub fn items(&self) -> Vec<Item> {
        let mut res = Vec::new();
        self.collect_items(&mut res);
        res.retain(|i| !i.is_command);
        res
    }

    fn collect_items(&self, res: &mut Vec<Item>) {
        match self {
            Meta::Empty => {}
            Meta::And(a, b) => {
                a.collect_items(res);
                b.collect_items(res);
            }
            Meta::Or(xs) => {
                for x in xs {
                    x.collect_items(res);
                }
            }
            Meta::Required(a) => a.collect_items(res),
            Meta::Many(a) => a.collect_items(res),
            Meta::Optional(a) => a.collect_items(res),
            Meta::Item(i) => res.push(i.clone()),
            Meta::Id => {}
        }
    }
}

impl std::fmt::Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Meta::Empty => Ok(()),
            Meta::And(a, b) => write!(f, "{} {}", a, b),
            Meta::Or(xs) => {
                let required = self.is_required();
                if required {
                    write!(f, "(")?;
                } else {
                    write!(f, "[")?;
                }
                for (ix, x) in xs.iter().enumerate() {
                    write!(f, "{}", x)?;
                    if ix + 1 < xs.len() {
                        write!(f, " | ")?;
                    }
                }
                if required {
                    write!(f, ")")
                } else {
                    write!(f, "]")
                }
            }
            Meta::Required(m) => write!(f, "({})", m),
            Meta::Optional(m) => write!(f, "[{}]", m),
            Meta::Many(m) => write!(f, "{}...", m),
            Meta::Item(i) => write!(f, "{}", i),
            Meta::Id => Ok(()),
        }
    }
}

impl From<Item> for Meta {
    fn from(item: Item) -> Self {
        Meta::Item(item)
    }
}

macro_rules! field {
    ($field:ident, $ty:ty) => {
        pub fn $field(mut self, $field: $ty) -> Self {
            self.$field = Some($field);
            self
        }
    };
}

#[derive(Clone)]
pub struct ParserInfo<T> {
    pub(crate) parse: Rc<dyn Fn(Args) -> Result<(T, Args), Error>>,
}
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub author: Option<&'static str>,
    pub version: Option<&'static str>,
    pub descr: Option<&'static str>,
    pub header: Option<&'static str>,
    pub footer: Option<&'static str>,
    pub usage: Option<&'static str>,
}

impl Info {
    field!(author, &'static str);
    field!(version, &'static str);
    field!(descr, &'static str);
    field!(header, &'static str);
    field!(footer, &'static str);
    field!(usage, &'static str);
}

/*
Missing: (-s|--source URI)

Usage: feed-dump [COMMAND | [-F FILTER] [-m|--machine-rdbl] | (-B|--benchmark) |
                   (-D|--dump-delays) | (-r|--raw-feed)] [-R|--reorder-feed]
                 [-c|--preserve-duplicates] (-s|--source URI)
                 [(-K|--skip-to-time TIME) | (-S|--skip-to-start)]
  Performs several types of manipulations on raw market data feed
*/

/*

 tt7% ./release/bin/feed-dump --help
Usage: feed-dump [COMMAND | [-F FILTER] [-m|--machine-rdbl] | (-B|--benchmark) |
                   (-D|--dump-delays) | (-r|--raw-feed)] [-R|--reorder-feed]
                 [-c|--preserve-duplicates] (-s|--source URI)
                 [(-K|--skip-to-time TIME) | (-S|--skip-to-start)]
  Performs several types of manipulations on raw market data feed

Available options:
  -F FILTER                dump only packets containing specific instrument,
                           e.g. -F 'opt kse-kospi200 2020-04w2 23000 put'
  -m,--machine-rdbl        Dump data in a format that's better suited for
                           further parsing
  -B,--benchmark           see benchmark command
  -D,--dump-delays         see delays command
  -h,--help                Show this help text
  --version                Show version and exit ignoring any other options

...................

Available commands:
  dump                     Decode and dump market data
                           Usual format: TIMESTAMP PACKET_INDICATOR DATA
                           where TIMESTAMP - UTC time when packet was received by the server
                           PACKET_INDICATOR - ^ and | show if current piece of data belongs

 */

impl Info {
    pub fn render_help(
        self,
        parser_meta: Meta,
        help_meta: Meta,
    ) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut res = String::new();

        match self.usage {
            Some(u) => write!(res, "{}\n", u)?,
            None => write!(res, "Usage: {}", parser_meta)?,
        }
        if let Some(t) = self.descr {
            write!(res, "\n{}\n", t)?;
        }
        if let Some(t) = self.header {
            write!(res, "\n{}\n", t)?;
        }
        let meta = Meta::and(parser_meta, help_meta);
        let items = &meta.items();

        let max_name_width = items.iter().map(|i| i.name_len()).max().unwrap_or(0);
        if !items.is_empty() {
            write!(res, "\nAvailable options:\n")?;
        }
        for i in items {
            if i.is_command {}
            match i.short {
                Some(c) => write!(res, "    -{}", c)?,
                None => write!(res, "       ")?,
            }
            if i.short.is_some() && i.long.is_some() {
                write!(res, ", ")?
            } else {
                write!(res, "  ")?
            }
            match (i.long, i.metavar) {
                (None, None) => write!(res, "{:ident$}", "", ident = max_name_width)?,
                (None, Some(m)) => write!(res, "{:ident$}", m, ident = max_name_width)?,
                (Some(l), None) => write!(res, "{:ident$}", l, ident = max_name_width)?,
                (Some(l), Some(m)) => write!(
                    res,
                    "{:ident$}",
                    format!("{} <{}>", l, m),
                    ident = max_name_width
                )?,
            }
            match i.help {
                Some(h) => write!(res, "{}\n", h)?,
                None => {
                    // strip unnecessary spaces inserted by previous writes
                    res.truncate(res.trim_end().len());
                    write!(res, "\n")?;
                }
            }
        }

        let commands = &meta.commands();
        if !commands.is_empty() {
            write!(res, "\nAvailable commands:\n")?;
        }
        let max_command_width = commands
            .iter()
            .map(|i| i.long.unwrap().len())
            .max()
            .unwrap_or(0);
        for c in commands {
            write!(
                res,
                "    {:indent$}",
                c.long.unwrap(),
                indent = max_command_width
            )?;
            if let Some(help) = c.help {
                write!(res, "  {}\n", help)?;
            } else {
                // strip unnecessary spaces inserted by previous writes
                res.truncate(res.trim_end().len());
            }
        }

        if let Some(t) = self.footer {
            write!(res, "{}\n\n", t)?;
        }
        Ok(res)
    }

    pub fn for_parser<T>(self, parser: Parser<T>) -> ParserInfo<T>
    where
        T: 'static + Clone,
    {
        #[derive(Clone)]
        enum I {
            Help,
            Version,
        }
        let p = move |i: Args| {
            let help = short('h')
                .long("help")
                .help("Prints help information")
                .req_flag(I::Help)
                .build();
            let ver = short('v')
                .long("version")
                .help("Prints version information")
                .req_flag(I::Version)
                .build();
            let hv = if self.version.is_some() {
                help.or_else(ver)
            } else {
                help
            };

            let check_unexpected = |(a, b): (T, Args)| match b.peek() {
                None => Ok((a, b)),
                Some(item) => Err(Error::Stderr(format!(
                    "{} is not expected in this context",
                    item
                ))),
            };

            let err = match (parser.parse)(i.clone()).and_then(check_unexpected) {
                Ok(r) => return Ok(r),
                Err(err) => err,
            };

            match (hv.clone().parse)(i) {
                Ok((I::Help, _)) => {
                    // TODO - why clone?
                    let msg = self
                        .clone()
                        .render_help(parser.meta.clone(), hv.meta)
                        .unwrap();
                    return Err(Error::Stdout(msg));
                }
                Ok((I::Version, _)) => {
                    if let Some(v) = self.version {
                        return Err(Error::Stdout(format!("Version: {}", v)));
                    } else {
                        unreachable!()
                    }
                }
                Err(_) => {}
            }
            Err(err)
        };
        ParserInfo { parse: Rc::new(p) }
    }
}
