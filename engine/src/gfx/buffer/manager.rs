use std::sync::Arc;
use crate::{gfx::buffer::WgpuBuffer, EngineError};


pub type WgpuBufferCacheType = crate::HashCache<Arc<WgpuBuffer>>;

/// Simple cache manager for `WgpuBuffer` resources keyed by `CacheKey`.
pub struct WgpuBufferManager {
    inner: WgpuBufferCacheType,
}

impl WgpuBufferManager {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl crate::CacheStorage<Arc<WgpuBuffer>> for WgpuBufferManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&Arc<WgpuBuffer>> {
        self.inner.get(key)
    }

    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.inner.contains_key(key)
    }

    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut Arc<WgpuBuffer>> {
        self.inner.get_mut(key)
    }

    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<WgpuBuffer>, EngineError>
    where
        F: FnOnce() -> Result<Arc<WgpuBuffer>, EngineError>,
    {
        self.inner.get_or_create(key, create_fn)
    }

    fn insert_resource(&mut self, key: crate::CacheKey, resource: Arc<WgpuBuffer>) {
        self.inner.insert(key, resource);
    }

    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<Arc<WgpuBuffer>> {
        self.inner.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<WgpuBuffer>>
    where
        WgpuBuffer: 'a,
    {
        self.inner.values()
    }
}
