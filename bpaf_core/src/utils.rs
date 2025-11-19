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
