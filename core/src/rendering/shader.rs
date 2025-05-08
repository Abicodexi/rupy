use crate::{CacheStorage, Managers};

pub struct Shader;
impl Shader {
    pub fn load(
        managers: &mut Managers,
        shader: &str,
    ) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        let start = std::time::Instant::now();
        let cache_key: crate::CacheKey = shader.into();

        if !managers.shader_manager.contains(&cache_key) {
            let module = crate::Asset::shader(managers, shader)?;
            managers
                .shader_manager
                .shaders
                .insert(cache_key.clone(), module.into());
        }
        crate::log_debug!(
            "[ShaderManager] Loaded shader `{}` in {:.2?}",
            cache_key.id,
            start.elapsed()
        );
        Ok(managers
            .shader_manager
            .get(&cache_key)
            .expect("Loading shader failed. Shader not found in manager cache after creation")
            .clone())
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
}

impl crate::CacheStorage<std::sync::Arc<wgpu::ShaderModule>> for ShaderManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
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
    ) -> &mut std::sync::Arc<wgpu::ShaderModule>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::ShaderModule>,
    {
        self.shaders.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::ShaderModule>) {
        self.shaders.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.shaders.remove(key);
    }
}
