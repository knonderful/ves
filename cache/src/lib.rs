use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A generic cache of entries.
///
/// # Generic types
/// * `T`: The entry type.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedCache<T> {
    /// A vector of cached entries.
    entries: Vec<T>,
    /// A hash map of entry hash values to indices into `entries`.
    #[cfg_attr(feature = "serde", serde(skip))]
    hashes: HashMap<u64, Vec<usize>>,
}

impl<T> IndexedCache<T> {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hashes: HashMap::new(),
        }
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<T> IndexedCache<T> where
    T: PartialEq + Hash + Clone,
{
    /// Offers a value.
    ///
    /// # Parameters
    /// * `value`: A [`Cow`] of the value to add. [`Cow::into_owned`] will be called if the value is not found in the cache.
    ///
    /// # Return
    /// The index of the entry.
    pub fn offer(&mut self, value: Cow<T>) -> usize {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(indices) = self.hashes.get_mut(&hash) {
            // We've seen this hash before, so we need to compare with the existing values of this hash
            indices.iter()
                // Look up the value for this index
                .map(|i| (i, &self.entries[*i]))
                // Compare the value
                .find(|(_, val)| *val == &*value)
                // Deref the index and ignore the value (since we're only interested in the index)
                .map(|(i, _)| *i)
                // Handle new entry
                .unwrap_or_else(|| {
                    let index = self.entries.len();
                    self.entries.push(value.into_owned());
                    indices.push(index);
                    index
                })
        } else {
            // This is a new hash, so we can just add it and update the hashes
            let index = self.entries.len();
            self.entries.push(value.into_owned());
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


/// We need a custom [`Deserialize`] implementation because every entry needs to be fed into `offer()` in order to build up our `hashes` and
/// we don't want to `hashes` to be a part of the (de)serialization.
#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for IndexedCache<T> where
    T: serde::Deserialize<'de> + PartialEq + Hash + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: serde::Deserializer<'de>
    {
        let visitor = IndexedCacheDeserializeVisitor(std::marker::PhantomData);
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(visitor)
        } else {
            deserializer.deserialize_seq(visitor)
        }
    }
}

/// A [`Visitor`] for deserialization.
#[cfg(feature = "serde")]
struct IndexedCacheDeserializeVisitor<T>(std::marker::PhantomData<T>);

#[cfg(feature = "serde")]
impl<'de, T> serde::de::Visitor<'de> for IndexedCacheDeserializeVisitor<T> where
    T: serde::Deserialize<'de> + PartialEq + Hash + Clone,
{
    type Value = IndexedCache<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a non-empty sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where
        A: serde::de::SeqAccess<'de>,
    {
        let mut cache = IndexedCache::<T>::new();
        while let Some(val) = seq.next_element()? {
            cache.offer(Cow::Owned(val));
        }
        Ok(cache)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where
        A: serde::de::MapAccess<'de>,
    {
        let mut cache = IndexedCache::<T>::new();
        while let Some((key, value)) = map.next_entry::<String, Vec<T>>()? {
            if key != "entries" {
                return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Other(&key), &"entries"));
            }

            for val in value {
                cache.offer(Cow::Owned(val));
            }
        }
        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::hash::{Hash, Hasher};
    use crate::IndexedCache;

    #[cfg_attr(feature = "serde", derive(serde::Deserialize), derive(serde::Serialize))]
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
        let mut cache = IndexedCache::<Val>::new();
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

        assert_eq!(cache.entries.len(), 4);
        let mut value_iter = cache.hashes.values();
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(2, value_iter.next().unwrap().len());
        assert_eq!(true, value_iter.next().is_none());
    }

    #[test]
    fn test_index() {
        let mut cache = IndexedCache::<Val>::new();
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

        assert_eq!(Val::new(0x1122334455667788, 120), cache[0]);
        assert_eq!(Val::new(0x1122334455667788, 240), cache[1]);
        assert_eq!(Val::new(0x8877665544332211, 120), cache[2]);
        assert_eq!(Val::new(0x8877665544332211, 240), cache[3]);
        assert_eq!(4, cache.len());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_json() {
        let mut cache = IndexedCache::<Val>::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 240);
        let val3 = Val::new(0x8877665544332211, 120);
        let val4 = Val::new(0x8877665544332211, 240);

        cache.offer(Cow::Owned(val1));
        cache.offer(Cow::Owned(val2));
        cache.offer(Cow::Owned(val3));
        cache.offer(Cow::Owned(val4));

        let str = serde_json::to_string(&cache).unwrap();
        assert_eq!(
            "{\"entries\":[{\"hash_seed\":1234605616436508552,\"data\":120},{\"hash_seed\":1234605616436508552,\"data\":240},{\"hash_seed\":9833440827789222417,\"data\":120},{\"hash_seed\":9833440827789222417,\"data\":240}]}",
            str
        )
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_json() {
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
        assert_eq!(0, expected.offer(Cow::Owned(Val::new(1234605616436508552, 120))));
        assert_eq!(1, expected.offer(Cow::Owned(Val::new(1234605616436508552, 240))));
        assert_eq!(2, expected.offer(Cow::Owned(Val::new(9833440827789222417, 120))));
        assert_eq!(3, expected.offer(Cow::Owned(Val::new(9833440827789222417, 240))));

        assert_eq!(expected, actual);
    }

    #[cfg(feature = "serde")]
    const BINCODE_DATA: [u8; 44] = [
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
        0x78, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        0x77, 0x88, 0x78, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0xf0,
    ];

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_bincode() {
        let mut cache = IndexedCache::<Val>::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 240);
        let val3 = Val::new(0x8877665544332211, 120);
        let val4 = Val::new(0x8877665544332211, 240);

        cache.offer(Cow::Owned(val1));
        cache.offer(Cow::Owned(val2));
        cache.offer(Cow::Owned(val3));
        cache.offer(Cow::Owned(val4));

        let data = bincode::serialize(&cache).unwrap();

        assert_eq!(&BINCODE_DATA, &data[..]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_bincode() {
        let mut expected = IndexedCache::<Val>::new();
        let val1 = Val::new(0x1122334455667788, 120);
        let val2 = Val::new(0x1122334455667788, 240);
        let val3 = Val::new(0x8877665544332211, 120);
        let val4 = Val::new(0x8877665544332211, 240);

        expected.offer(Cow::Owned(val1));
        expected.offer(Cow::Owned(val2));
        expected.offer(Cow::Owned(val3));
        expected.offer(Cow::Owned(val4));

        let actual = bincode::deserialize(&BINCODE_DATA).unwrap();
        assert_eq!(&expected, &actual);
    }
}
