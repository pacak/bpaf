//! Free monoid on a set of annotated strings
//!
//! This module implements a free monoid on a set of annotated string slices along with some extra
//! tools to operate on it. For typical usage you can check `typical_usage` function since
//! abstractions used here are scary, not exposed to the end user and hidden from the doctest.

#[test]
fn typical_usage() {
    let mut m = FreeMonoid::<char>::default();
    m.push_str('a', "string ")
        .push_str('b', "more string")
        .push('d', '!');

    let mut r = String::new();
    for (a, slice) in &m {
        r += &format!("{}: {:?}; ", a, slice);
    }
    assert_eq!("a: \"string \"; b: \"more string\"; d: \"!\"; ", r);
}

/// A Free Monoid on set of annotated string slices
///
/// Where identity element is `FreeMonoid::default` and binary operation is `+`
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)] // FreeMonoid is a thing, Free is not
pub struct FreeMonoid<T> {
    payload: String,
    labels: Vec<(std::ops::Range<usize>, T)>,
    /// Merge adjacent fields with the same metadata together during insertion
    pub squash: bool,
}

impl<T> Default for FreeMonoid<T> {
    fn default() -> Self {
        Self {
            payload: String::new(),
            labels: Vec::new(),
            squash: false,
        }
    }
}

impl<T> FreeMonoid<T> {
    /// Length of stored text in bytes
    ///
    /// Does not account for space required to render the annotations
    pub(crate) fn payload_size(&self) -> usize {
        self.payload.len()
    }

    /// Append an annotated string slice
    pub(crate) fn push_str(&mut self, meta: T, payload: &str) -> &mut Self
    where
        T: PartialEq,
    {
        let r = self.payload.len();
        self.payload.push_str(payload);
        if let Some((prev_range, prev_meta)) = self.labels.last_mut() {
            if self.squash && prev_meta == &meta {
                prev_range.end = self.payload.len();
                return self;
            }
        }
        self.labels.push((r..self.payload.len(), meta));
        self
    }

    /// Append an annotated char slice
    pub(crate) fn push(&mut self, meta: T, payload: char) -> &mut Self
    where
        T: PartialEq,
    {
        let r = self.payload.len();
        self.payload.push(payload);
        if let Some((prev_range, prev_meta)) = self.labels.last_mut() {
            if self.squash && prev_meta == &meta {
                prev_range.end = self.payload.len();
                return self;
            }
        }
        self.labels.push((r..self.payload.len(), meta));
        self
    }

    /// Iterate over annotated fragments
    pub(crate) fn iter(&self) -> AnnotatedSlicesIter<T> {
        AnnotatedSlicesIter {
            current: 0,
            items: self,
        }
    }
}

impl<'a, T> Extend<(T, &'a str)> for FreeMonoid<T>
where
    T: PartialEq,
{
    fn extend<I: IntoIterator<Item = (T, &'a str)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.push_str(k, v);
        }
    }
}

impl<'a, T> IntoIterator for &'a FreeMonoid<T> {
    type Item = (&'a T, &'a str);
    type IntoIter = AnnotatedSlicesIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone> std::ops::Add<&Self> for FreeMonoid<T> {
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T: Clone> std::ops::AddAssign<&Self> for FreeMonoid<T> {
    fn add_assign(&mut self, rhs: &Self) {
        self.payload.push_str(&rhs.payload);
        let len = self.payload.len();
        self.labels.extend(
            rhs.labels
                .iter()
                .map(|(range, label)| (range.start + len..range.end + len, label.clone())),
        );
    }
}

impl<T: PartialEq> std::ops::AddAssign<(T, &str)> for FreeMonoid<T> {
    fn add_assign(&mut self, rhs: (T, &str)) {
        self.push_str(rhs.0, rhs.1);
    }
}

impl<T: PartialEq> std::ops::AddAssign<(T, char)> for FreeMonoid<T> {
    fn add_assign(&mut self, rhs: (T, char)) {
        self.push(rhs.0, rhs.1);
    }
}

/// Iterate over annotated string slices contained in a [`FreeMonoid`].
///
/// Create with [`FreeMonoid::iter`]
pub struct AnnotatedSlicesIter<'a, T> {
    current: usize,
    items: &'a FreeMonoid<T>,
}

impl<'a, T> Iterator for AnnotatedSlicesIter<'a, T> {
    type Item = (&'a T, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (range, label) = self.items.labels.get(self.current)?;
        self.current += 1;
        Some((label, &self.items.payload[range.clone()]))
    }
}
