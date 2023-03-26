use crate::{
    args::Arg,
    item::{Item, ShortLong},
    meta_help::{Long, Short},
    Args, Meta,
};

/// Looks for potential typos
pub(crate) fn suggest(args: &Args, meta: &Meta) -> Option<String> {
    let arg = args.peek()?;

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
        } else if l > 0 && l < usize::MAX {
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
            Self::LongCmd(s) => write!(f, "command or positional: `{}`", w_err!(s)),
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
        damerau_levenshtein(&expected.to_string(), &actual.to_string()),
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
        | Meta::Decorated(meta, _, _) => {
            collect_suggestions(arg, meta, variants, at_top_level);
        }
        Meta::Skip => {}
    }
}

/// Damerau-Levenshtein distance function
///
/// returns `usize::MAX` if there's no common characters at all mostly to avoid
/// confusing error messages - "you typed 'foo', maybe you ment 'bar'" where
/// 'foo' and 'bar' don't have anything in common
fn damerau_levenshtein(a: &str, b: &str) -> usize {
    #![allow(clippy::many_single_char_names)]
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    let mut d = vec![0; (a_len + 1) * (b_len + 1)];

    let ix = |ib, ia| a_len * ia + ib;

    for i in 0..=a_len {
        d[ix(i, 0)] = i;
    }

    for j in 0..=b_len {
        d[ix(0, j)] = j;
    }

    let mut pa = '\0';
    let mut pb = '\0';
    for (i, ca) in a.chars().enumerate() {
        let i = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let j = j + 1;
            let cost = usize::from(ca != cb);
            d[ix(i, j)] = (d[ix(i - 1, j)] + 1)
                .min(d[ix(i, j - 1)] + 1)
                .min(d[ix(i - 1, j - 1)] + cost);
            if i > 1 && j > 1 && ca == pb && cb == pa {
                d[ix(i, j)] = d[ix(i, j)].min(d[ix(i - 2, j - 2)] + 1);
            }
            pb = cb;
        }
        pa = ca;
    }

    let diff = d[ix(a_len, b_len)];

    if diff >= a_len.max(b_len) {
        usize::MAX
    } else {
        diff
    }
}
