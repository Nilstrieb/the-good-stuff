use std::hash::{BuildHasher, Hash};

pub mod simple_open_addressing;

pub trait HashMapFamily {
    type Map<K, V, S>: HashMap<K, V, S>;
}

pub trait HashMap<K, V, S>: IntoIterator<Item = (K, V)> {
    fn with_hasher(state: S) -> Self;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash,
        S: BuildHasher;

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq + Hash,
        S: BuildHasher;
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, BuildHasherDefault, Hasher, RandomState};

    use super::{HashMap, HashMapFamily};

    #[derive(Default)]
    struct CollidingHasher;
    impl Hasher for CollidingHasher {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _bytes: &[u8]) {}
    }

    pub(super) fn run_tests<M>()
    where
        M: HashMapFamily,
    {
        let mk_str = || M::Map::<&str, &str, _>::with_hasher(RandomState::new());

        let m = mk_str();
        assert_eq!(m.get(&"uwu"), None);
        assert_eq!(m.get(&"uwu"), None);

        let mut m = mk_str();
        m.insert("hello", "world");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.len(), 1);
        m.insert("aaa", "yes");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.get(&"aaa"), Some(&"yes"));
        assert_eq!(m.len(), 2);

        let mut m = mk_str();
        m.insert("hello", "world");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.len(), 1);
        m.insert("hello", "no");
        assert_eq!(m.get(&"hello"), Some(&"no"));
        assert_eq!(m.len(), 1);

        for count in [1, 10, 100, 1000, 10_000, 100_000] {
            test_many::<M, _>(count, RandomState::new());
        }
        test_many::<M, _>(1000, BuildHasherDefault::<CollidingHasher>::default());
    }

    fn test_many<M: HashMapFamily, H: BuildHasher>(count: usize, h: H) {
        let mut m = M::Map::with_hasher(h);

        for i in 0..count {
            m.insert(i, i);
        }

        let mut found = vec![false; count];
        for (k, v) in m.into_iter() {
            assert_eq!(k, v);
            assert!(!found[k], "duplicate element");
            found[k] = true;
        }
        for (i, found) in found.iter().enumerate() {
            assert!(found, "element {i} was lost");
        }
    }
}
