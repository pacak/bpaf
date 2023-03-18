use crate::{
    args::Arg,
    item::{Item, ShortLong},
    meta_help::{Long, Short},
    Args, Error, Meta,
};

pub(crate) fn should_suggest(err: &Error) -> bool {
    match err {
        Error::Message(_) => true,
        Error::ParseFailure(_) => false,
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
pub(crate) fn suggest(args: &Args, meta: &Meta) -> Option<String> {
    let arg = match args.peek() {
        Some(arg) => arg,
        None => return None,
    };

    if args.items.iter().filter(|&a| a == arg).count() > 1 {
        // args contains more than one copy of unexpected item. Either user specified
        // several of those or parser accepts only limited number of them.
        // Or a different branch handles them. Give up and produce a default
        // "not expected in this context" error
        return None;
    }

    let mut variants = Vec::new();
    collect_suggestions(arg, meta, &mut variants, true);

    variants.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some((l, (actual, expected))) = variants.pop() {
        if let (0, I::Nested(cmd)) = (l, expected) {
            let best = format!(
                "{:?} is not valid in this context, did you mean to pass it to command \"{}\"?",
                actual,
                w_flag!(cmd),
            );
            return Some(best);
        } else if l > 0 {
            let best = format!(
                "No such {:?}, did you mean `{}`?",
                actual,
                w_flag!(expected)
            );
            return Some(best);
        }
    }
    None
}

#[derive(Copy, Clone)]
enum I<'a> {
    ShortFlag(char),
    LongFlag(&'a str),
    Ambiguity(&'a str),
    ShortCmd(char),
    LongCmd(&'a str),
    Nested(&'a str),
}

// human readable
impl std::fmt::Debug for I<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortFlag(s) => write!(f, "flag: `{}`", w_err!(Short(*s))),
            Self::LongFlag(s) => write!(f, "flag: `{}`", w_err!(Long(s))),
            Self::ShortCmd(s) => write!(f, "command alias: `{}`", w_err!(s)),
            Self::LongCmd(s) => write!(f, "command: `{}`", w_err!(s)),
            Self::Ambiguity(s) => write!(f, "flag: {} (with one dash)", w_err!(s)),
            Self::Nested(s) => write!(f, "command {}", w_err!(s)),
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
            Self::LongCmd(s) | Self::Ambiguity(s) | Self::Nested(s) => f.write_str(s),
        }
    }
}

fn ins<'a>(expected: I<'a>, actual: I<'a>, variants: &mut Vec<(usize, (I<'a>, I<'a>))>) {
    variants.push((
        levenshtein(&expected.to_string(), &actual.to_string()),
        (actual, expected),
    ));
}

fn inner_item<'a>(
    arg: &'a Arg,
    item: &'a Item,
    variants: &mut Vec<(usize, (I<'a>, I<'a>))>,
    at_top_level: bool,
) {
    let actual: I = match arg {
        Arg::Short(s, _, _) => I::ShortFlag(*s),
        Arg::Long(s, _, _) => I::LongFlag(s.as_str()),
        Arg::Word(w) | Arg::PosWord(w) => match &w.to_str() {
            Some(s) => I::LongCmd(s),
            None => return,
        },
        Arg::Ambiguity(_, os) => {
            if let Some(s) = os.to_str() {
                I::Ambiguity(s)
            } else {
                return;
            }
        }
    };
    match item {
        Item::Positional { .. } => {}
        Item::Command {
            name, short, meta, ..
        } => {
            if at_top_level {
                let mut inner = Vec::new();
                collect_suggestions(arg, meta, &mut inner, false);
                if let Some((0, _)) = inner.first() {
                    variants.push((0, (actual, I::Nested(name))));
                }
            }
            ins(I::LongCmd(name), actual, variants);
            if let Some(s) = short {
                ins(I::ShortCmd(*s), actual, variants);
            }
        }
        Item::Flag { name, .. } | Item::Argument { name, .. } | Item::MultiArg { name, .. } => {
            match name {
                ShortLong::Short(s) => ins(I::ShortFlag(*s), actual, variants),
                ShortLong::Long(l) => ins(I::LongFlag(l), actual, variants),
                ShortLong::ShortLong(s, l) => {
                    ins(I::ShortFlag(*s), actual, variants);
                    ins(I::LongFlag(l), actual, variants);
                }
            }
        }
    }
}

fn collect_suggestions<'a>(
    arg: &'a Arg,
    meta: &'a Meta,
    variants: &mut Vec<(usize, (I<'a>, I<'a>))>,
    at_top_level: bool,
) {
    match meta {
        Meta::And(xs) | Meta::Or(xs) => {
            for x in xs {
                collect_suggestions(arg, x, variants, at_top_level);
            }
        }
        Meta::Item(item) => inner_item(arg, item, variants, at_top_level),
        Meta::HideUsage(meta)
        | Meta::Optional(meta)
        | Meta::Many(meta)
        | Meta::Decorated(meta, _) => {
            collect_suggestions(arg, meta, variants, at_top_level);
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
