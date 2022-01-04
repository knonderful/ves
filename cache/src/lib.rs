use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Index;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{Error, MapAccess, Unexpected, Visitor};

/// A generic cache of entries.
///
/// # Generic types
/// * `T`: The entry type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IndexedCache<T> {
    /// A vector of cached values.
    entries: Vec<T>,
    /// A hash map of hash values for `V` to indices into `values`.
    #[serde(skip)]
    hashes: HashMap<u64, Vec<usize>>,
}

/// We need a custom [`Deserialize`] implementation because every entry needs to be fed into `offer()` in order to build up our `hashes` and
/// we don't want to `hashes` to be a part of the (de)serialization.
impl<'de, T> Deserialize<'de> for IndexedCache<T> where
    T: Deserialize<'de> + PartialEq + Hash,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(IndexedCacheDeserializeVisitor(PhantomData))
    }
}

/// A [`Visitor`] for deserialization.
struct IndexedCacheDeserializeVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for IndexedCacheDeserializeVisitor<T> where
    T: Deserialize<'de> + PartialEq + Hash,
{
    type Value = IndexedCache<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a non-empty sequence")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where
        A: MapAccess<'de>,
    {
        let mut cache = IndexedCache::<T>::new();
        while let Some((key, value)) = map.next_entry::<String, Vec<T>>()? {
            if key != "entries" {
                return Err(A::Error::invalid_value(Unexpected::Other(&key), &"entries"));
            }

            for val in value {
                cache.offer(val);
            }
        }
        Ok(cache)
    }
}

impl<T> IndexedCache<T> {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hashes: HashMap::new(),
        }
    }

    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.entries.len()
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
                .map(|i| (i, &self.entries[*i]))
                // Compare the value
                .find(|(_, val)| *val == &value)
                // Deref the index and ignore the value (since we're only interested in the index)
                .map(|(i, _)| *i)
                // Handle new entry
                .unwrap_or_else(|| {
                    let index = self.entries.len();
                    self.entries.push(value);
                    indices.push(index);
                    index
                })
        } else {
            // This is a new hash, so we can just add it and update the hashes
            let index = self.entries.len();
            self.entries.push(value);
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
        &self.entries[index]
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{Hash, Hasher};
    use crate::IndexedCache;

    #[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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

        assert_eq!(cache.entries.len(), 4);
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

    #[test]
    fn test_serialize() {
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

        let str = serde_json::to_string(&cache).unwrap();
        assert_eq!(
            "{\"entries\":[{\"hash_seed\":1234605616436508552,\"data\":120},{\"hash_seed\":1234605616436508552,\"data\":240},{\"hash_seed\":9833440827789222417,\"data\":120},{\"hash_seed\":9833440827789222417,\"data\":240}]}",
            str
        )
    }

    #[test]
    fn test_deserialize() {
        const INPUT: &'static str = r#"
            {
              "entries": [
                {
                  "hash_seed": 1234605616436508552,
                  "data": 120
                },
                {
                  "hash_seed": 1234605616436508552,
                  "data": 240
                },
                {
                  "hash_seed": 9833440827789222417,
                  "data": 120
                },
                {
                  "hash_seed": 9833440827789222417,
                  "data": 240
                }
              ]
            }
        "#;

        let actual: IndexedCache<Val> = serde_json::from_str(INPUT).unwrap();

        let mut expected = IndexedCache::new();
        assert_eq!(0, expected.offer(Val::new(1234605616436508552, 120)));
        assert_eq!(1, expected.offer(Val::new(1234605616436508552, 240)));
        assert_eq!(2, expected.offer(Val::new(9833440827789222417, 120)));
        assert_eq!(3, expected.offer(Val::new(9833440827789222417, 240)));

        assert_eq!(expected, actual);
    }
}
