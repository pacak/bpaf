use crate::{
    item::ShortLong,
    meta_help::{HelpItem, HelpItems},
    Meta, State,
};

#[derive(Debug, Copy, Clone)]
pub(crate) enum Variant {
    CommandLong(&'static str),
    Flag(ShortLong),
}

#[derive(Debug)]
pub(crate) enum Suggestion {
    Variant(Variant),
    /// expected --foo, actual -foo
    MissingDash(&'static str),
    /// expected -f, actual --f
    ExtraDash(char),
    Nested(String, Variant),
}

/// Looks for potential typos
#[inline(never)]
pub(crate) fn suggest(args: &State, meta: &Meta) -> Option<(usize, Suggestion)> {
    let (ix, arg) = args.items_iter().next()?;

    // suggesting typos for parts of group of short names (-vvv, typo in third v) would be strange
    if arg.os_str().is_empty() {
        return None;
    }
    // don't try to suggest fixes for typos in strictly positional items
    if matches!(arg, crate::args::Arg::PosWord(_)) {
        return None;
    }
    // it also should be a printable name
    let actual = arg.to_string();

    // all the help items one level deep
    let mut hi = HelpItems::default();
    hi.append_meta(meta);

    // this will be used to avoid reallocations on scannign
    let mut nested = HelpItems::default();

    // while scanning keep the closest match
    let mut best_match = None;
    let mut best_dist = usize::MAX;
    let mut improve = |dist, val| {
        if best_dist > dist && dist > 0 && dist < 4 {
            best_dist = dist;
            best_match = Some(val);
        }
    };

    let mut nest = None;

    for item in &hi.items {
        match item {
            HelpItem::Command { name, meta, .. } => {
                // command can result in 2 types of suggestions:
                // - typo in a short or a long name
                // - there is a nested command that matches perfectly - try using that
                let distance = damerau_levenshtein(&actual, name);
                improve(distance, Variant::CommandLong(name));

                // scan nested items and look for exact matches only
                nested.items.clear();
                nested.append_meta(meta);
                for item in &nested.items {
                    match item {
                        HelpItem::Command { name: nname, .. } => {
                            if *nname == actual {
                                nest = Some((name, Variant::CommandLong(nname)));
                            }
                        }
                        HelpItem::Flag { name: nname, .. }
                        | HelpItem::Argument { name: nname, .. } => {
                            if *nname == &actual {
                                nest = Some((name, Variant::Flag(*nname)));
                            }
                        }
                        HelpItem::DecorSuffix { .. }
                        | HelpItem::GroupStart { .. }
                        | HelpItem::GroupEnd { .. }
                        | HelpItem::Positional { .. }
                        | HelpItem::AnywhereStart { .. }
                        | HelpItem::AnywhereStop { .. }
                        | HelpItem::Any { .. } => {}
                    }
                }
            }
            HelpItem::Flag { name, .. } | HelpItem::Argument { name, .. } => {
                if let Some(long) = name.as_long() {
                    let distance = damerau_levenshtein(&actual, &format!("--{}", long));
                    improve(distance, Variant::Flag(*name));
                }
                if let Some(short) = name.as_short() {
                    if let Some(act) = actual.strip_prefix("--") {
                        let mut tmp = [0u8; 4];
                        if act == short.encode_utf8(&mut tmp) {
                            return Some((ix, Suggestion::ExtraDash(short)));
                        }
                    }
                }
            }
            HelpItem::Positional { .. }
            | HelpItem::DecorSuffix { .. }
            | HelpItem::GroupStart { .. }
            | HelpItem::GroupEnd { .. }
            | HelpItem::AnywhereStart { .. }
            | HelpItem::AnywhereStop { .. }
            | HelpItem::Any { .. } => {}
        }
    }

    if let Some((&name, variant)) = nest {
        Some((ix, Suggestion::Nested(name.to_string(), variant)))
    } else {
        // skip confusing errors
        if best_dist == usize::MAX {
            return None;
        }
        let best_match = best_match?;

        // handle missing single dash typos separately
        if let Variant::Flag(n) = best_match {
            if let Some(long) = n.as_long() {
                if actual.strip_prefix('-') == Some(long) {
                    return Some((ix, Suggestion::MissingDash(long)));
                }
            }
        }
        Some((ix, Suggestion::Variant(best_match)))
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

    if diff >= a_len.min(b_len) {
        usize::MAX
    } else {
        diff
    }
}
