use wgpu::ShaderModule;

use crate::{log_debug, log_error, AssetLoader, CacheKey, CacheStorage, EngineError, HashCache};
use std::sync::Arc;

pub struct ShaderManager {
    shaders: HashCache<Arc<wgpu::ShaderModule>>,
    asset_loader: Arc<AssetLoader>,
}

impl ShaderManager {
    pub fn new(asset_loader: Arc<AssetLoader>) -> Self {
        Self {
            shaders: HashCache::new(),
            asset_loader,
        }
    }
    pub fn reload_shader(&mut self, name: &str) {
        let start = std::time::Instant::now();
        match self.asset_loader.load_shader(name) {
            Ok(module) => {
                let arc_module = Arc::new(module);
                self.shaders.insert(CacheKey::new(name), arc_module);
                let elapsed = start.elapsed();
                log_debug!(
                    "[ShaderManager] Reloaded shader `{}` in {:.2?}",
                    name,
                    elapsed
                );
            }
            Err(e) => {
                log_error!("Shader reload error: {:?}", e);
            }
        }
    }

    pub fn get<K: Into<CacheKey>>(&self, name: K) -> Option<&Arc<wgpu::ShaderModule>> {
        self.shaders.get(&name.into())
    }

    pub fn get_or_create<K, F>(&mut self, name: K, create_fn: F) -> &mut Arc<ShaderModule>
    where
        F: FnOnce() -> Result<Arc<ShaderModule>, EngineError>,
        K: Into<CacheKey>,
    {
        let cache_key: CacheKey = name.into();

        self.shaders.get_or_create(cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create shader module");
            let elapsed = start.elapsed();
            log_debug!(
                "[ShaderManager] Loaded shader `{}` in {:.2?}",
                cache_key.id,
                elapsed
            );
            module
        })
    }
}
