pub struct ShaderManager {
    shaders: crate::HashCache<std::sync::Arc<wgpu::ShaderModule>>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: crate::HashCache::new(),
        }
    }
    pub fn reload_shader(&mut self, asset_loader: &crate::AssetLoader, name: &str) {
        let start = std::time::Instant::now();
        match asset_loader.load_shader(name) {
            Ok(module) => {
                let arc_module = std::sync::Arc::new(module);
                self.shaders.insert(crate::CacheKey::new(name), arc_module);
                let elapsed = start.elapsed();
                crate::log_debug!(
                    "[ShaderManager] Reloaded shader `{}` in {:.2?}",
                    name,
                    elapsed
                );
            }
            Err(e) => {
                crate::log_error!("Shader reload error: {:?}", e);
            }
        }
    }

    pub fn get<K: Into<crate::CacheKey>>(
        &self,
        name: K,
    ) -> Option<&std::sync::Arc<wgpu::ShaderModule>> {
        self.shaders.get(&name.into())
    }

    pub fn get_or_create<K, F>(
        &mut self,
        name: K,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::ShaderModule>
    where
        F: FnOnce() -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError>,
        K: Into<crate::CacheKey>,
    {
        let cache_key: crate::CacheKey = name.into();

        crate::CacheStorage::get_or_create(&mut self.shaders, cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create shader module");
            let elapsed = start.elapsed();
            crate::log_debug!(
                "[ShaderManager] Loaded shader `{}` in {:.2?}",
                cache_key.id,
                elapsed
            );
            module
        })
    }
}
