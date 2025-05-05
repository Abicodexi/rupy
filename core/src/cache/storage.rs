use std::collections::HashMap;

use super::key::CacheKey;

pub trait CacheStorage<R> {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&R>;
    fn contains(&self, key: &CacheKey) -> bool;
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut R>;
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut R
    where
        F: FnOnce() -> R;
    fn insert(&mut self, key: CacheKey, resource: R);
    fn remove(&mut self, key: &CacheKey);
}

pub type HashCache<R> = HashMap<CacheKey, R>;

impl<R> CacheStorage<R> for HashCache<R> {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&R> {
        self.get(&key.into())
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut R> {
        self.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut R
    where
        F: FnOnce() -> R,
    {
        self.entry(key.clone()).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: R) {
        self.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.remove(key);
    }
}
