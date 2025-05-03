use crate::{log_debug, CacheKey, CacheStorage, EngineError, HashCache};
use std::sync::Arc;

pub struct PipelineManager {
    render_pipelines: HashCache<Arc<wgpu::RenderPipeline>>,
    compute_pipelines: HashCache<Arc<wgpu::ComputePipeline>>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            render_pipelines: HashCache::new(),
            compute_pipelines: HashCache::new(),
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

    pub fn get_render_pipeline<K: Into<CacheKey>>(
        &self,
        name: K,
    ) -> Option<&Arc<wgpu::RenderPipeline>> {
        self.render_pipelines.get(&name.into())
    }
    pub fn get_compute_pipeline<K: Into<CacheKey>>(
        &self,
        name: K,
    ) -> Option<&Arc<wgpu::ComputePipeline>> {
        self.compute_pipelines.get(&name.into())
    }
    pub fn get_or_create_render_pipeline<K, F>(
        &mut self,
        name: K,
        create_fn: F,
    ) -> &mut Arc<wgpu::RenderPipeline>
    where
        F: FnOnce() -> Result<Arc<wgpu::RenderPipeline>, EngineError>,
        K: Into<CacheKey>,
    {
        let cache_key: CacheKey = name.into();

        self.render_pipelines.get_or_create(cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create render pipeline");
            let elapsed = start.elapsed();
            log_debug!(
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
    ) -> &mut Arc<wgpu::ComputePipeline>
    where
        F: FnOnce() -> Result<Arc<wgpu::ComputePipeline>, EngineError>,
        K: Into<CacheKey>,
    {
        let cache_key: CacheKey = name.into();

        self.compute_pipelines.get_or_create(cache_key.clone(), || {
            let start = std::time::Instant::now();
            let module = create_fn().expect("Failed to create compute pipelien");
            let elapsed = start.elapsed();
            log_debug!(
                "[PipelineManager] Loaded compute pipeline `{}` in {:.2?}",
                cache_key.id,
                elapsed
            );
            module
        })
    }
}
