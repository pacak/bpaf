/// Clean the vector and decouple the lifetime reuse the capacity
pub(crate) fn reuse_vec<U, V>(mut v: Vec<U>) -> Vec<V> {
    use core::mem::size_of;
    const {
        assert!(size_of::<U>() == size_of::<V>());
        assert!(align_of::<U>() == align_of::<V>());
    }
    v.clear();
    v.into_iter().map(|_| unreachable!()).collect()
}

/// A small vector that can hold 1 item without heap allocation
///
/// TODO - benchmark?
#[derive(Clone)]
pub(crate) struct Vec1<T>(Vec1Int<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for Vec1<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Vec1").field(&self.as_slice()).finish()
    }
}

#[derive(Clone)]
enum Vec1Int<T> {
    One(T),
    Vec(Vec<T>),
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
        *self = if let Vec1(Vec1Int::Vec(items)) = self
            && items.is_empty()
        {
            Vec1(Vec1Int::One(val))
        } else {
            let mut dummy = Vec1::default();

            std::mem::swap(&mut dummy, self);
            Vec1(match dummy.0 {
                Vec1Int::One(m) => Vec1Int::Vec(vec![m, val]),
                Vec1Int::Vec(mut items) => {
                    items.push(val);
                    Vec1Int::Vec(items)
                }
            })
        }
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        match &self.0 {
            Vec1Int::One(x) => std::slice::from_ref(x),
            Vec1Int::Vec(items) => items.as_slice(),
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
