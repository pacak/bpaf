/// A small vector that can hold 1 item without heap allocation
///
/// TODO - benchmark?
#[derive(Clone)]
pub(crate) struct Vec1<T>(Vec1Int<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for Vec1<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[derive(Clone)]
enum Vec1Int<T> {
    One(T),
    Vec(Vec<T>),
}

impl<T> From<Vec<T>> for Vec1<T> {
    fn from(value: Vec<T>) -> Self {
        Vec1(Vec1Int::Vec(value))
    }
}

impl<T> Default for Vec1<T> {
    fn default() -> Self {
        Self(Vec1Int::Vec(Vec::new()))
    }
}

impl<T> Vec1<T> {
    pub(crate) fn new(val: T) -> Self {
        Vec1(Vec1Int::One(val))
    }

    pub(crate) fn push(&mut self, val: T) {
        match std::mem::replace(self, Vec1(Vec1Int::Vec(Vec::new()))) {
            Vec1(Vec1Int::One(first)) => {
                *self = Vec1(Vec1Int::Vec(vec![first, val]));
            }
            Vec1(Vec1Int::Vec(items)) if items.is_empty() => {
                *self = Vec1(Vec1Int::One(val));
            }
            Vec1(Vec1Int::Vec(mut items)) => {
                items.push(val);
                *self = Vec1(Vec1Int::Vec(items));
            }
        }
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        match &self.0 {
            Vec1Int::One(x) => std::slice::from_ref(x),
            Vec1Int::Vec(items) => items.as_slice(),
        }
    }
}
impl<T: Ord> Vec1<T> {
    pub(crate) fn sort(&mut self) {
        if let Vec1Int::Vec(items) = &mut self.0 {
            items.sort()
        }
    }
}

impl<T> std::ops::Add for Vec1<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec1(match (self.0, rhs.0) {
            (Vec1Int::One(a), Vec1Int::One(b)) => Vec1Int::Vec(vec![a, b]),
            (Vec1Int::Vec(empty), v) | (v, Vec1Int::Vec(empty)) if empty.is_empty() => v,
            (Vec1Int::One(v), Vec1Int::Vec(mut items)) => {
                items.insert(0, v);
                Vec1Int::Vec(items)
            }
            (Vec1Int::Vec(mut items), Vec1Int::One(v)) => {
                items.push(v);
                Vec1Int::Vec(items)
            }
            (Vec1Int::Vec(mut items), Vec1Int::Vec(extra)) => {
                items.extend(extra);
                Vec1Int::Vec(items)
            }
        })
    }
}

/// Damerau-Levenshtein distance function
pub(crate) fn damerau_levenshtein(a: &str, b: &str) -> f32 {
    #![allow(clippy::many_single_char_names)]
    // working with bytes should give results that is close enough while avoiding bloat
    // from utf8 parsing machinery
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (a_len, b_len) = (a.len(), b.len());

    let mut d = vec![0; (a_len + 1) * (b_len + 1)];

    let ix = |ib, ia| a_len * ia + ib;

    for i in 0..=a_len {
        d[ix(i, 0)] = i;
    }

    for j in 0..=b_len {
        d[ix(0, j)] = j;
    }

    let mut pa = 0;
    let mut pb = 0;
    for (i, ca) in a.iter().copied().enumerate() {
        for (j, cb) in b.iter().copied().enumerate() {
            let cost = usize::from(ca != cb);
            d[ix(i + 1, j + 1)] = (d[ix(i, j + 1)] + 1)
                .min(d[ix(i + 1, j)] + 1)
                .min(d[ix(i, j + 1 - 1)] + cost);
            if i > 0 && j > 0 && ca == pb && cb == pa {
                d[ix(i + 1, j + 1)] = d[ix(i + 1, j + 1)].min(d[ix(i - 1, j - 1)] + 1);
            }
            pb = cb;
        }
        pa = ca;
    }

    d[ix(a_len, b_len)] as f32 / a_len.max(b_len) as f32
}
