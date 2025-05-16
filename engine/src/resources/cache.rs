pub trait CacheStorage<R> {
    fn get(&self, key: &crate::CacheKey) -> Option<&R>;
    fn contains(&self, key: &crate::CacheKey) -> bool;
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut R>;
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut R
    where
        F: FnOnce() -> R;
    fn insert(&mut self, key: crate::CacheKey, resource: R);
    fn remove(&mut self, key: &crate::CacheKey) -> Option<R>;
}

pub type HashCache<R> = std::collections::HashMap<crate::CacheKey, R>;

impl<R> CacheStorage<R> for HashCache<R> {
    fn get(&self, key: &crate::CacheKey) -> Option<&R> {
        self.get(&key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut R> {
        self.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut R
    where
        F: FnOnce() -> R,
    {
        self.entry(key.clone()).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: R) {
        self.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) -> Option<R> {
        self.remove(key)
    }
}
