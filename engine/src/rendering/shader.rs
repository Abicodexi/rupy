use std::sync::Arc;

use crate::{AssetLoader, CacheStorage, EngineError};

pub struct Shader;
impl Shader {
    pub fn load(shader: &str) -> Result<wgpu::ShaderModule, crate::EngineError> {
        let binding = crate::gpu_global::get_global_gpu();
        let gpu = binding
            .read()
            .map_err(|e| crate::EngineError::PoisonError(format!("{}", e.to_string())))?;

        let path = AssetLoader::base_path().join("shaders").join(shader);

        let shader_source = std::fs::read_to_string(&path)?;
        let shader_module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
        Ok(shader_module)
    }
}
pub struct ShaderManager {
    pub shaders: crate::HashCache<std::sync::Arc<wgpu::ShaderModule>>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: crate::HashCache::new(),
        }
    }
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        shader: &str,
    ) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        let cache_key = crate::CacheKey::from(shader);

        if !self.contains_resource(&cache_key) {
            let path = AssetLoader::base_path().join("shaders").join(shader);

            let shader_source = std::fs::read_to_string(&path)?;
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
            self.insert_resource(cache_key.clone(), shader_module.into());
        }
        Ok(self.get_resource(&cache_key).unwrap().clone())
    }
}

impl crate::CacheStorage<std::sync::Arc<wgpu::ShaderModule>> for ShaderManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.get(key)
    }

    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.shaders.contains_key(key)
    }
    fn get_mut(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<&mut std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<wgpu::ShaderModule>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::ShaderModule>, EngineError>,
    {
        self.shaders.get_or_create(key, create_fn)
    }
    fn insert_resource(
        &mut self,
        key: crate::CacheKey,
        resource: std::sync::Arc<wgpu::ShaderModule>,
    ) {
        self.shaders.insert(key, resource);
    }
    fn remove_resource(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<wgpu::ShaderModule>>
    where
        std::sync::Arc<wgpu::ShaderModule>: 'a,
    {
        self.shaders.values()
    }
}
