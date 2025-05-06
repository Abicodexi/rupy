pub struct Shader;
impl Shader {
    pub fn load(
        managers: &mut crate::Managers,
        shader: &str,
    ) -> Result<std::sync::Arc<wgpu::ShaderModule>, crate::EngineError> {
        let start = std::time::Instant::now();
        let cache_key: crate::CacheKey = shader.into();

        if !crate::CacheStorage::contains(&managers.shader_manager, &cache_key) {
            let module = crate::AssetLoader::load_shader(managers, shader)?;
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
        Ok(
            crate::CacheStorage::get(&managers.shader_manager, &cache_key)
                .expect("Loading shader failed. Shader not found in manager cache after creation")
                .clone(),
        )
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

pub struct ShaderHotReloader;

impl ShaderHotReloader {
    pub fn watch(
        watcher_tx: &std::sync::Arc<crossbeam::channel::Sender<crate::ApplicationEvent>>,
    ) -> Result<(), crate::EngineError> {
        let shader_dir = crate::asset_dir()?.join("shaders");
        let reload_map = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
            String,
            std::time::Instant,
        >::new()));
        let tx = watcher_tx.clone();
        let _ = crate::AssetWatcher::new(shader_dir, {
            let reload_map = std::sync::Arc::clone(&reload_map);
            move |event| {
                for path in event.paths.iter() {
                    let filename = path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or_default()
                        .to_string();

                    let mut map = reload_map.lock().unwrap();
                    let now = std::time::Instant::now();
                    let last = map.entry(filename.clone()).or_insert(now);

                    if now.duration_since(*last) > std::time::Duration::from_millis(2) {
                        *last = now;
                        if let Err(e) =
                            tx.send(crate::ApplicationEvent::ShaderLoad(filename.into()))
                        {
                            crate::log_error!("Error sending shader hot reload event: {}", e);
                        };
                    }
                }
            }
        });
        crate::log_info!("Exiting shader watcher loop");
        Ok(())
    }
}
