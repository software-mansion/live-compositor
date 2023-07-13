use std::{collections::HashMap, hash::Hash, sync::Mutex};

pub struct SyncHashMap<K, V>(std::sync::Mutex<HashMap<K, V>>);

impl<K, V> SyncHashMap<K, V>
where
    K: Eq + PartialEq + Hash,
{
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn insert(&self, key: K, value: V) {
        let mut map = self.0.lock().unwrap();
        map.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut map = self.0.lock().unwrap();
        map.remove(key)
    }
}

impl<K, V> SyncHashMap<K, V>
where
    K: Eq + PartialEq + Hash,
    V: Clone,
{
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        let map = self.0.lock().unwrap();
        map.get(key).map(Clone::clone)
    }
}

impl<K, V> Default for SyncHashMap<K, V>
where
    K: Eq + PartialEq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}
