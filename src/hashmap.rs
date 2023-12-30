use std::{
    hash::{BuildHasher, Hash, RandomState},
    vec,
};

type Entry<K, V> = Option<(K, V)>;

pub struct HashMap<K, V, S = RandomState> {
    buckets: Vec<Entry<K, V>>,
    filled: usize,
    s: S,
}

impl<K: Eq + Hash, V> HashMap<K, V, RandomState> {
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> HashMap<K, V, S> {
    pub fn with_hasher(state: S) -> Self {
        Self {
            buckets: Vec::new(),
            filled: 0,
            s: state,
        }
    }

    pub fn len(&self) -> usize {
        self.filled
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.len() == 0
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if self.is_empty() {
            return None;
        }
        let bucket = self.bucket_of_elem(&key);

        let result = self.buckets[bucket..]
            .iter()
            .take_while(|elem| elem.is_some())
            .find(|elem| matches!(elem, Some((elem_key, _)) if elem_key == key));

        if let Some(Some((_, value))) = result {
            Some(value)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.filled >= self.buckets.len() {
            self.grow();
        }
        loop {
            let bucket = self.bucket_of_elem(&key);
            let bucket = self.buckets[bucket..].iter_mut().find(|bucket| {
                bucket.is_none() || matches!(bucket, Some((elem_key, _)) if *elem_key == key)
            });
            if let Some(bucket) = bucket {
                if bucket.is_none() {
                    self.filled += 1;
                }
                *bucket = Some((key, value));
                return;
            } else {
                self.grow();
            }
        }
    }

    fn bucket_of_elem(&self, key: &K) -> usize {
        assert_ne!(self.buckets.len(), 0, "cannot compute bucket of empty map");
        let hash = self.s.hash_one(&key) as usize;
        hash % self.buckets.len()
    }

    fn grow(&mut self) {
        let len = self.buckets.len();
        let new = if len == 0 { 8 } else { len * 2 };
        let old = IntoIter::new(std::mem::take(&mut self.buckets));
        let new_buckets = (0..new).map(|_| None).collect();
        self.buckets = new_buckets;
        self.extend(old);
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> Extend<(K, V)> for HashMap<K, V, S> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        iter.into_iter()
            .for_each(|(key, value)| self.insert(key, value));
    }
}

pub struct IntoIter<K, V> {
    buckets: std::iter::FilterMap<vec::IntoIter<Entry<K, V>>, fn(Entry<K, V>) -> Option<(K, V)>>,
}

impl<K, V> IntoIter<K, V> {
    fn new(buckets: Vec<Entry<K, V>>) -> Self {
        IntoIter {
            buckets: buckets.into_iter().filter_map(std::convert::identity),
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.buckets.next()
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.buckets)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, BuildHasherDefault, Hasher, RandomState};

    use super::HashMap;

    #[test]
    fn get_empty() {
        let m = HashMap::<&str, ()>::new();
        assert_eq!(m.get(&"uwu"), None);
        assert_eq!(m.get(&"uwu"), None);
    }

    #[test]
    fn insert() {
        let mut m = HashMap::new();
        m.insert("hello", "world");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.len(), 1);
        m.insert("aaa", "yes");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.get(&"aaa"), Some(&"yes"));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn overriding() {
        let mut m = HashMap::new();
        m.insert("hello", "world");
        assert_eq!(m.get(&"hello"), Some(&"world"));
        assert_eq!(m.len(), 1);
        m.insert("hello", "no");
        assert_eq!(m.get(&"hello"), Some(&"no"));
        assert_eq!(m.len(), 1);
    }

    #[derive(Default)]
    struct CollidingHasher;
    impl Hasher for CollidingHasher {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _bytes: &[u8]) {}
    }

    fn test_many<H: BuildHasher>(count: usize, h: H) {
        let mut m = HashMap::with_hasher(h);

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

    #[test]
    fn many_elements() {
        for count in [1, 10, 100, 1000, 10_000, 100_000] {
            test_many(count, RandomState::new());
        }
    }

    #[test]
    fn many_many_collisions() {
        test_many(5000, BuildHasherDefault::<CollidingHasher>::default());
    }
}
