use super::{HashMap, HashMapFamily};
use std::{
    hash::{BuildHasher, Hash, RandomState},
    vec,
};

type Entry<K, V> = Option<(K, V)>;

pub struct SimpleOAHashMap<K, V, S = RandomState> {
    buckets: Vec<Entry<K, V>>,
    filled: usize,
    s: S,
}

impl<K: Eq + Hash, V> SimpleOAHashMap<K, V, RandomState> {
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> SimpleOAHashMap<K, V, S> {
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

impl<K: Eq + Hash, V, S: BuildHasher> Extend<(K, V)> for SimpleOAHashMap<K, V, S> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        iter.into_iter()
            .for_each(|(key, value)| drop(self.insert(key, value)));
    }
}

impl<K, V, S> super::HashMap<K, V, S> for SimpleOAHashMap<K, V, S> {
    fn with_hasher(state: S) -> Self {
        Self {
            buckets: Vec::new(),
            filled: 0,
            s: state,
        }
    }

    fn len(&self) -> usize {
        self.filled
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
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

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
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
                let before = std::mem::replace(bucket, Some((key, value)));
                return before.map(|(_, v)| v);
            } else {
                self.grow();
            }
        }
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

impl<K, V, S> IntoIterator for SimpleOAHashMap<K, V, S> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.buckets)
    }
}

pub struct SimpleOAHashMapFamily;
impl HashMapFamily for SimpleOAHashMapFamily {
    type Map<K, V, S> = SimpleOAHashMap<K, V, S>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn do_tests() {
        crate::hashmaps::tests::run_tests::<super::SimpleOAHashMapFamily>();
    }
}
