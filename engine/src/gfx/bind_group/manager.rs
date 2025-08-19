use std::sync::Arc;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource};

use crate::{CacheKey, EngineError, log_error, HashCache, CacheStorage, Texture, TextureManager};

/// Caches bind groups keyed by a `CacheKey` (usually a texture path/handle).
pub struct BindGroupManager {
    bind_groups: HashCache<Arc<BindGroup>>,
}

impl BindGroupManager {
    pub fn new() -> Self {
        Self { bind_groups: HashCache::new() }
    }

    /// Get or create a bind group for a *2D texture + sampler* layout
    /// (e.g., `textures::diffuse_layout`) by texture identifier.
    ///
    /// Loads the texture via `TextureManager` if missing.
    pub fn texture_bind_group(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        texture_id: &str,
        layout: &BindGroupLayout,
    ) -> Option<Arc<BindGroup>> {
        let key = CacheKey::from(texture_id);

        if let Some(bg) = self.bind_groups.get(&key) {
            return Some(bg.clone());
        }

        if !textures.contains_resource(&key) {
            if let Err(e) = textures.load(queue, device, texture_id) {
                log_error!(
                    "Failed to load texture '{}' for bind group: {}",
                    texture_id,
                    e
                );
                return None;
            }
        }

        let tex = textures.get_resource(&key)?;
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&tex.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&tex.sampler),
            },
        ];

        let bg: Arc<BindGroup> = device
            .create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("{} bind group", texture_id)),
                layout,
                entries: &entries,
            })
            .into();

        self.bind_groups.insert(key.clone(), bg.clone());
        Some(bg)
    }

    /// When you already have a `Texture` (e.g., from a material),
    /// build (and cache) a 2D texture+sampler bind group with the given cache key.
    pub fn from_texture(
        &mut self,
        device: &wgpu::Device,
        cache_key: CacheKey,
        texture: &Texture,
        layout: &BindGroupLayout,
        label: Option<&str>,
    ) -> Arc<BindGroup> {
        if let Some(bg) = self.bind_groups.get(&cache_key) {
            return bg.clone();
        }

        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&texture.sampler),
            },
        ];

        let bg: Arc<BindGroup> = device
            .create_bind_group(&BindGroupDescriptor {
                label: label.or(Some("texture bind group")),
                layout,
                entries: &entries,
            })
            .into();

        self.bind_groups.insert(cache_key, bg.clone());
        bg
    }
}

impl CacheStorage<Arc<BindGroup>> for BindGroupManager {
    fn get_resource(&self, key: &CacheKey) -> Option<&Arc<BindGroup>> {
        self.bind_groups.get(key)
    }

    fn contains_resource(&self, key: &CacheKey) -> bool {
        self.bind_groups.contains_key(key)
    }

    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<BindGroup>> {
        self.bind_groups.get_mut(key)
    }

    fn get_or_create<F>(
        &mut self,
        key: CacheKey,
        create_fn: F,
    ) -> Result<Arc<BindGroup>, EngineError>
    where
        F: FnOnce() -> Result<Arc<BindGroup>, EngineError>,
    {
        self.bind_groups.get_or_create(key, create_fn)
    }

    fn insert_resource(&mut self, key: CacheKey, resource: Arc<BindGroup>) {
        self.bind_groups.insert(key, resource);
    }

    fn remove_resource(&mut self, key: &CacheKey) -> Option<Arc<BindGroup>> {
        self.bind_groups.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<BindGroup>>
    where
        Arc<BindGroup>: 'a,
    {
        self.bind_groups.values()
    }
}

