pub struct ShaderManager {
    pub shaders: crate::HashCache<std::sync::Arc<wgpu::ShaderModule>>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: crate::HashCache::new(),
        }
    }
    pub fn reload_shader(&mut self, device: &wgpu::Device, name: &str) {
        let start = std::time::Instant::now();
        match crate::AssetLoader::load_shader(device, name) {
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

pub struct ShaderHotReload {}

impl ShaderHotReload {
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
