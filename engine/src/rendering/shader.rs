pub struct Shader;
impl Shader {
    pub const DEFAULT: &str = "v_normal.wgsl";

    pub fn load(shader: &str) -> Result<wgpu::ShaderModule, crate::EngineError> {
        let binding = crate::GPU::get();
        let gpu = binding
            .read()
            .map_err(|e| crate::EngineError::PoisonError(format!("{}", e.to_string())))?;

        let path = crate::Asset::base_path().join("shaders").join(shader);

        let shader_source = std::fs::read_to_string(&path)?;
        let shader_module = gpu
            .device()
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
        let start = std::time::Instant::now();

        if !crate::CacheStorage::contains(self, &cache_key) {
            let path = crate::Asset::base_path().join("shaders").join(shader);

            let shader_source = std::fs::read_to_string(&path)?;
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
            crate::CacheStorage::insert(self, cache_key.clone(), shader_module.into());
        }
        crate::log_debug!("Loaded in {:.2?}", start.elapsed());
        Ok(crate::CacheStorage::get(self, &cache_key).unwrap().clone())
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
    fn remove(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.remove(key)
    }
}
