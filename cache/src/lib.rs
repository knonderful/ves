use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A generic cache of values.
///
/// # Generic types
/// * `T`: The contained type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedCache<T> {
    /// A vector of cached values.
    values: Vec<T>,
    /// A hash map of hash values for `V` to indices into `values`.
    hashes: HashMap<u64, Vec<usize>>,
}

impl<T> IndexedCache<T> {
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
}

impl<T> IndexedCache<T> where
    T: PartialEq + Hash,
{
    /// Offers a value.
    ///
    /// # Parameters
    /// * `value`: The value to add.
    ///
    /// # Return
    /// The index of the entry.
    pub fn offer(&mut self, value: T) -> usize {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(indices) = self.hashes.get_mut(&hash) {
            // We've seen this hash before, so we need to compare with the existing values of this hash
            indices.iter()
                // Look up the value for this index
                .map(|i| (i, &self.values[*i]))
                // Compare the value
                .find(|(_, val)| *val == &value)
                // Deref the index and ignore the value (since we're only interested in the index)
                .map(|(i, _)| *i)
                // Handle new entry
                .unwrap_or_else(|| {
                    let index = self.values.len();
                    self.values.push(value);
                    indices.push(index);
                    index
                })
        } else {
            // This is a new hash, so we can just add it and update the hashes
            let index = self.values.len();
            self.values.push(value);
            if self.hashes.insert(hash, vec![index]).is_some() {
                // This can only happen with a local programming error
                panic!("Expected no element to be pre-existing for hash {}.", hash);
            }
            index
        }
    }
}

impl<T> Index<usize> for IndexedCache<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{Hash, Hasher};
    use crate::IndexedCache;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct Val {
        hash: u64,
        other: u8,
    }

    impl Val {
        fn new(hash: u64, other: u8) -> Self {
            Self { hash, other }
        }
    }

    impl Hash for Val {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u64(self.hash)
        }
    }

    #[test]
    fn test_offer() {
        let mut cache = IndexedCache::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 120);
        let val3 = Val::new(0x1122334455667788, 240);
        let val4 = Val::new(0x8877665544332211, 120);
        let val5 = Val::new(0x8877665544332211, 240);
        let val6 = Val::new(0x8877665544332211, 120);

        assert_eq!(cache.offer(val1), 0usize);
        assert_eq!(cache.offer(val2), 0usize);
        assert_eq!(cache.offer(val3), 1usize);
        assert_eq!(cache.offer(val4), 2usize);
        assert_eq!(cache.offer(val5), 3usize);
        assert_eq!(cache.offer(val6), 2usize);
        assert_eq!(cache.offer(val2), 0usize);
        assert_eq!(cache.offer(val3), 1usize);

        assert_eq!(cache.values.len(), 4);
        let mut value_iter = cache.hashes.values();
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(true, value_iter.next().is_none());
    }

    #[test]
    fn test_index() {
        let mut cache = IndexedCache::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 120);
        let val3 = Val::new(0x1122334455667788, 240);
        let val4 = Val::new(0x8877665544332211, 120);
        let val5 = Val::new(0x8877665544332211, 240);
        let val6 = Val::new(0x8877665544332211, 120);

        cache.offer(val1);
        cache.offer(val2);
        cache.offer(val3);
        cache.offer(val4);
        cache.offer(val5);
        cache.offer(val6);
        cache.offer(val2);
        cache.offer(val3);

        assert_eq!(Val::new(0x1122334455667788, 120), cache[0]);
        assert_eq!(Val::new(0x1122334455667788, 240), cache[1]);
        assert_eq!(Val::new(0x8877665544332211, 120), cache[2]);
        assert_eq!(Val::new(0x8877665544332211, 240), cache[3]);
        assert_eq!(4, cache.len());
    }
}
