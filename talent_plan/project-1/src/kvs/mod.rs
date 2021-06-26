use std::collections::HashMap;
use std::hash::Hash;

pub struct KvStore<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: K) -> Option<V> {
        match self.map.get(&key) {
            Some(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: K, val: V) {
        self.map.insert(key, val);
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
       self.map.remove(&key)
    }
}
