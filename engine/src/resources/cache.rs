use crate::EngineError;

pub trait CacheStorage<R: Clone> {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&R>;
    fn all<'a>(&'a self) -> impl Iterator<Item = &'a R>
    where
        R: 'a;
    fn contains_resource(&self, key: &crate::CacheKey) -> bool;
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut R>;
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> Result<R, EngineError>
    where
        F: FnOnce() -> Result<R, EngineError>;
    fn insert_resource(&mut self, key: crate::CacheKey, resource: R);
    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<R>;
}

pub type HashCache<R> = std::collections::HashMap<crate::CacheKey, R>;

impl<R: Clone> CacheStorage<R> for HashCache<R> {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&R> {
        self.get(&key)
    }
    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut R> {
        self.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> Result<R, EngineError>
    where
        F: FnOnce() -> Result<R, EngineError>,
    {
        if let Some(v) = self.get(&key) {
            Ok(v.clone())
        } else {
            let item = create_fn()?;
            self.insert(key, item.clone());
            Ok(item)
        }
    }
    fn insert_resource(&mut self, key: crate::CacheKey, resource: R) {
        self.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<R> {
        self.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a R>
    where
        R: 'a,
    {
        self.values()
    }
}
