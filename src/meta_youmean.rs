use crate::{
    args::Arg,
    item::{Item, ShortLong},
    Args, Error, Meta,
};

pub(crate) fn should_suggest(err: &Error) -> bool {
    match err {
        Error::Stdout(_) => false,
        Error::Stderr(_) => true,
        Error::Missing(xs) => {
            let mut hi = crate::meta_help::HelpItems::default();
            for x in xs.iter() {
                hi.classify_item(x);
            }
            hi.flgs.is_empty() && hi.psns.is_empty()
        }
    }
}

/// Looks for potential typos
pub(crate) fn suggest(args: &Args, meta: &Meta) -> Result<(), Error> {
    if args.tainted {
        return Ok(());
    }

    let arg = match args.peek() {
        Some(arg) => arg,
        None => return Ok(()),
    };

    if args.items.iter().filter(|&a| a == arg).count() > 1 {
        // args contains more than one copy of unexpected item. Either user specified
        // several of those or parser accepts only limited number of them.
        // Or a different branch handles them. Give up and produce a default
        // "not expected in this context" error
        return Ok(());
    }

    let mut variants = Vec::new();
    inner(arg, meta, &mut variants);

    variants.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some((l, best)) = variants.pop() {
        if l > 0 {
            return Err(Error::Stderr(best));
        }
    }
    Ok(())
}

#[derive(Copy, Clone)]
enum I<'a> {
    ShortFlag(char),
    LongFlag(&'a str),
    ShortCmd(char),
    LongCmd(&'a str),
}

// human readable
impl std::fmt::Debug for I<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortFlag(s) => write!(f, "flag: `-{}`", s),
            Self::LongFlag(s) => write!(f, "flag: `--{}`", s),
            Self::ShortCmd(s) => write!(f, "command alias: `{}`", s),
            Self::LongCmd(s) => write!(f, "command: `{}`", s),
        }
    }
}

// used for levenshtein distance calculation
impl std::fmt::Display for I<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        match self {
            Self::ShortFlag(s) => write!(f, "-{}", s),
            Self::LongFlag(s) => write!(f, "--{}", s),
            Self::ShortCmd(s) => f.write_char(*s),
            Self::LongCmd(s) => f.write_str(s),
        }
    }
}

fn ins(expected: I, actual: I, variants: &mut Vec<(usize, String)>) {
    variants.push((
        levenshtein(&expected.to_string(), &actual.to_string()),
        format!("No such {:?}, did you mean `{}`?", actual, expected),
    ));
}

fn inner_item(arg: &Arg, item: &Item, variants: &mut Vec<(usize, String)>) {
    let actual: I = match arg {
        Arg::Short(s, _, _) => I::ShortFlag(*s),
        Arg::Long(s, _, _) => I::LongFlag(s.as_str()),
        Arg::Word(w) | Arg::PosWord(w) => match &w.to_str() {
            Some(s) => I::LongCmd(s),
            None => return,
        },
        // shouldn't be reachable
        Arg::Ambiguity(_, _) => return,
    };
    match item {
        Item::Positional { .. } => {}
        Item::Command { name, short, .. } => {
            ins(I::LongCmd(name), actual, variants);
            if let Some(s) = short {
                ins(I::ShortCmd(*s), actual, variants);
            }
        }
        Item::Flag { name, .. } | Item::Argument { name, .. } => match name {
            ShortLong::Short(s) => ins(I::ShortFlag(*s), actual, variants),
            ShortLong::Long(l) => ins(I::LongFlag(l), actual, variants),
            ShortLong::ShortLong(s, l) => {
                ins(I::ShortFlag(*s), actual, variants);
                ins(I::LongFlag(l), actual, variants);
            }
        },
    }
}

fn inner(arg: &Arg, meta: &Meta, variants: &mut Vec<(usize, String)>) {
    match meta {
        Meta::And(xs) | Meta::Or(xs) => {
            for x in xs {
                inner(arg, x, variants);
            }
        }
        Meta::Item(item) => inner_item(arg, item, variants),
        Meta::Optional(meta) | Meta::Many(meta) | Meta::Decorated(meta, _) => {
            inner(arg, meta, variants);
        }
        Meta::Skip => {}
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut result = 0;
    let mut cache = a.chars().enumerate().map(|i| i.0 + 1).collect::<Vec<_>>();
    let mut distance_a;
    let mut distance_b;

    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }
    result
}
