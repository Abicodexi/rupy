use std::sync::Arc;

use wgpu::{Sampler, SamplerDescriptor};

use crate::{CacheKey, EngineError, HashCache, CacheStorage};
use super::presets::{SamplerPreset, create_sampler_from_preset};

/// Very small cache for samplers.
/// Keys are `CacheKey` (e.g., "sampler:linear_mipmap_repeat" or custom ids).
pub struct SamplerManager {
    cache: HashCache<Arc<Sampler>>,
}

impl SamplerManager {
    pub fn new() -> Self {
        Self { cache: HashCache::new() }
    }

    /// Get or create a sampler by preset.
    pub fn preset(
        &mut self,
        device: &wgpu::Device,
        preset: SamplerPreset,
    ) -> Arc<Sampler> {
        let key = CacheKey::from(preset.as_string());
        if let Some(s) = self.cache.get(&key) {
            return s.clone();
        }
        let sampler = create_sampler_from_preset(device, preset, Some(&preset.as_string()));
        let sampler = Arc::new(sampler);
        self.cache.insert(key, sampler.clone());
        sampler
    }

    /// Get or create a sampler from an explicit descriptor, keyed with a provided CacheKey.
    /// (Use this for ad-hoc/custom samplers.)
    pub fn custom_with_key(
        &mut self,
        device: &wgpu::Device,
        key: CacheKey,
        desc: &SamplerDescriptor,
    ) -> Arc<Sampler> {
        if let Some(s) = self.cache.get(&key) {
            return s.clone();
        }
        let sampler = Arc::new(device.create_sampler(desc));
        self.cache.insert(key, sampler.clone());
        sampler
    }

    /// Like `custom_with_key`, but generates a key string for you (be careful: identical descs
    /// with different labels will produce different keys).
    pub fn custom(
        &mut self,
        device: &wgpu::Device,
        name: &str,
        desc: &SamplerDescriptor,
    ) -> Arc<Sampler> {
        let key = CacheKey::from(format!("sampler:custom:{name}"));
        self.custom_with_key(device, key, desc)
    }
}

impl CacheStorage<Arc<Sampler>> for SamplerManager {
    fn get_resource(&self, key: &CacheKey) -> Option<&Arc<Sampler>> {
        self.cache.get(key)
    }

    fn contains_resource(&self, key: &CacheKey) -> bool {
        self.cache.contains_key(key)
    }

    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<Sampler>> {
        self.cache.get_mut(key)
    }

    fn get_or_create<F>(
        &mut self,
        key: CacheKey,
        create_fn: F,
    ) -> Result<Arc<Sampler>, EngineError>
    where
        F: FnOnce() -> Result<Arc<Sampler>, EngineError>,
    {
        self.cache.get_or_create(key, create_fn)
    }

    fn insert_resource(&mut self, key: CacheKey, resource: Arc<Sampler>) {
        self.cache.insert(key, resource);
    }

    fn remove_resource(&mut self, key: &CacheKey) -> Option<Arc<Sampler>> {
        self.cache.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<Sampler>>
    where
        Arc<Sampler>: 'a,
    {
        self.cache.values()
    }
}

