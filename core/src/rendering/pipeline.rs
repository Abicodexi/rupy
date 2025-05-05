pub struct PipelineManager {
    pub render_pipelines: crate::HashCache<std::sync::Arc<wgpu::RenderPipeline>>,
    pub compute_pipelines: crate::HashCache<std::sync::Arc<wgpu::ComputePipeline>>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            render_pipelines: crate::HashCache::new(),
            compute_pipelines: crate::HashCache::new(),
        }
    }
    // pub fn reload_shader(&mut self, name: &str) {
    //     let start = std::time::Instant::now();
    //     match self.asset_loader.load_shader(name) {
    //         Ok(module) => {
    //             let arc_module = Arc::new(module);
    //             self.shaders.insert(CacheKey::new(name), arc_module);
    //             let elapsed = start.elapsed();
    //             log_debug!(
    //                 "[ShaderManager] Reloaded shader `{}` in {:.2?}",
    //                 name,
    //                 elapsed
    //             );
    //         }
    //         Err(e) => {
    //             log_error!("Shader reload error: {:?}", e);
    //         }
    //     }
    // }

    pub fn get_render_pipeline<K: Into<crate::CacheKey>>(
        &self,
        name: K,
    ) -> Option<&std::sync::Arc<wgpu::RenderPipeline>> {
        self.render_pipelines.get(&name.into())
    }
    pub fn get_compute_pipeline<K: Into<crate::CacheKey>>(
        &self,
        name: K,
    ) -> Option<&std::sync::Arc<wgpu::ComputePipeline>> {
        self.compute_pipelines.get(&name.into())
    }
    pub fn get_or_create_render_pipeline<K, F>(
        &mut self,
        name: K,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::RenderPipeline>
    where
        F: FnOnce() -> Result<std::sync::Arc<wgpu::RenderPipeline>, crate::EngineError>,
        K: Into<crate::CacheKey>,
    {
        let cache_key: crate::CacheKey = name.into();

        crate::CacheStorage::get_or_create(&mut self.render_pipelines, cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create render pipeline");
            let elapsed = start.elapsed();
            crate::log_debug!(
                "[PipelineManager] Loaded render pipeline `{}` in {:.2?}",
                cache_key.id,
                elapsed
            );
            module
        })
    }
    pub fn get_or_create_compute_pipeline<K, F>(
        &mut self,
        name: K,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::ComputePipeline>
    where
        F: FnOnce() -> Result<std::sync::Arc<wgpu::ComputePipeline>, crate::EngineError>,
        K: Into<crate::CacheKey>,
    {
        let cache_key: crate::CacheKey = name.into();

        crate::CacheStorage::get_or_create(&mut self.compute_pipelines, cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create compute pipelien");
            let elapsed = start.elapsed();
            crate::log_debug!(
                "[PipelineManager] Loaded compute pipeline `{}` in {:.2?}",
                cache_key.id,
                elapsed
            );
            module
        })
    }
}
