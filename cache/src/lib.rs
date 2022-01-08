use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A trait for retrieving the index into a collection of a type.
pub trait AsIndex {
    /// Retrieves the index value.
    fn as_index(&self) -> usize;
}

/// A trait for creating an instance from an index.
pub trait FromIndex {
    /// Creates an instance from the provided index.
    fn from_index(index: usize) -> Self;
}

impl AsIndex for usize {
    fn as_index(&self) -> usize {
        *self
    }
}

impl FromIndex for usize {
    fn from_index(index: usize) -> Self {
        index
    }
}

/// A mutable [`Vec`]-based cache.
///
/// # Generic types
/// * `T`: The element type. This type should implement [`PartialEq`], [`Hash`] and [`Clone`].
/// * `K`: The key type. This type should implement [`Copy`], [`AsIndex`] and [`FromIndex`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VecCacheMut<T, K = usize> {
    /// A vector of cached values.
    values: Vec<T>,
    /// A hash map of value hash values to indices into `values`.
    hashes: HashMap<u64, Vec<K>>,
}

impl<T, K> VecCacheMut<T, K> {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            hashes: HashMap::new(),
        }
    }

    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Consumes this instance and returns the values.
    pub fn consume(self) -> Vec<T> {
        self.values
    }
}

impl<T, K> VecCacheMut<T, K> where
    T: PartialEq + Hash + Clone,
    K: Copy + AsIndex + FromIndex,
{
    /// Offers a value.
    ///
    /// # Parameters
    /// * `value`: A [`Cow`] of the value to add. [`Cow::into_owned`] will be called if the value is not found in the cache.
    ///
    /// # Return
    /// The key.
    pub fn offer(&mut self, value: Cow<T>) -> K {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(indices) = self.hashes.get_mut(&hash) {
            // We've seen this hash before, so we need to compare with the existing values of this hash
            indices.iter()
                // Look up the value for this index
                .map(|i| (i, &self.values[i.as_index()]))
                // Compare the value
                .find(|(_, val)| *val == &*value)
                // Deref the index and ignore the value (since we're only interested in the index)
                .map(|(i, _)| *i)
                // Handle new value
                .unwrap_or_else(|| {
                    let index = K::from_index(self.values.len());
                    self.values.push(value.into_owned());
                    indices.push(index);
                    index
                })
        } else {
            // This is a new hash, so we can just add it and update the hashes
            let index = K::from_index(self.values.len());
            self.values.push(value.into_owned());
            if self.hashes.insert(hash, vec![index]).is_some() {
                // This can only happen with a local programming error
                panic!("Expected no element to be pre-existing for hash {}.", hash);
            }
            index
        }
    }
}

impl<T, K> Index<K> for VecCacheMut<T, K> where
    K: AsIndex,
{
    type Output = T;

    fn index(&self, index: K) -> &Self::Output {
        &self.values[index.as_index()]
    }
}

#[cfg(test)]
mod test_vec_cache_mut {
    use std::borrow::Cow;
    use std::hash::{Hash, Hasher};
    use crate::VecCacheMut;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct Val {
        hash_seed: u64,
        data: u8,
    }

    impl Val {
        fn new(hash_seed: u64, data: u8) -> Self {
            Self { hash_seed, data }
        }
    }

    impl Hash for Val {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u64(self.hash_seed)
        }
    }

    #[test]
    fn test_offer() {
        let mut cache = VecCacheMut::<Val>::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 120);
        let val3 = Val::new(0x1122334455667788, 240);
        let val4 = Val::new(0x8877665544332211, 120);
        let val5 = Val::new(0x8877665544332211, 240);
        let val6 = Val::new(0x8877665544332211, 120);

        assert_eq!(cache.offer(Cow::Owned(val1)), 0usize);
        assert_eq!(cache.offer(Cow::Owned(val2)), 0usize);
        assert_eq!(cache.offer(Cow::Owned(val3)), 1usize);
        assert_eq!(cache.offer(Cow::Owned(val4)), 2usize);
        assert_eq!(cache.offer(Cow::Owned(val5)), 3usize);
        assert_eq!(cache.offer(Cow::Owned(val6)), 2usize);
        assert_eq!(cache.offer(Cow::Owned(val2)), 0usize);
        assert_eq!(cache.offer(Cow::Owned(val3)), 1usize);

        assert_eq!(cache.values.len(), 4);
        let mut value_iter = cache.hashes.values();
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(true, value_iter.next().is_none());
    }

    #[test]
    fn test_index() {
        let mut cache = VecCacheMut::<Val>::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 120);
        let val3 = Val::new(0x1122334455667788, 240);
        let val4 = Val::new(0x8877665544332211, 120);
        let val5 = Val::new(0x8877665544332211, 240);
        let val6 = Val::new(0x8877665544332211, 120);

        cache.offer(Cow::Owned(val1));
        cache.offer(Cow::Owned(val2));
        cache.offer(Cow::Owned(val3));
        cache.offer(Cow::Owned(val4));
        cache.offer(Cow::Owned(val5));
        cache.offer(Cow::Owned(val6));
        cache.offer(Cow::Owned(val2));
        cache.offer(Cow::Owned(val3));

        assert_eq!(Val::new(0x1122334455667788, 120), cache[0usize]);
        assert_eq!(Val::new(0x1122334455667788, 240), cache[1usize]);
        assert_eq!(Val::new(0x8877665544332211, 120), cache[2usize]);
        assert_eq!(Val::new(0x8877665544332211, 240), cache[3usize]);
        assert_eq!(4, cache.len());
    }
}
