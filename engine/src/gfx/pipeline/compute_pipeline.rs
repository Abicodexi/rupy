use std::sync::Arc;

use crate::{AssetService, CacheKey, EngineError};

/// Create a compute pipeline from a CS shader.
pub fn create_compute_pipeline<'a>(
    service: &AssetService,
    c_shader: &str,
    layout: wgpu::PipelineLayout,
    entry_point: Option<&'a str>,
    label: &str,
) -> Result<wgpu::ComputePipeline, EngineError> {
    service.load_shader(c_shader);

    let shader = service.get_shader(&CacheKey::from(c_shader)).unwrap();

    let pipeline = service.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        cache: Default::default(),
        module: &shader,
        entry_point,
        compilation_options: Default::default(),
    });

    Ok(pipeline)
}

/// Cache manager for compute pipelines.
pub struct ComputePipelineManager {
    pipelines: crate::HashCache<Arc<wgpu::ComputePipeline>>,
}

impl ComputePipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: crate::HashCache::new(),
        }
    }

    pub fn create_pipeline<'a>(
        &mut self,
        service: &AssetService,
        c_shader: &str,
        layout: wgpu::PipelineLayout,
        entry_point: Option<&'a str>,
        key: CacheKey,
        label: &str,
    ) -> Result<(), EngineError> {
        if !self.pipelines.contains_key(&key) {
            let pipeline = create_compute_pipeline(service, c_shader, layout, entry_point, label)?;
            self.pipelines.insert(key, pipeline.into());
        }
        Ok(())
    }
}

impl crate::CacheStorage<Arc<wgpu::ComputePipeline>> for ComputePipelineManager {
    fn get_resource(&self, key: &CacheKey) -> Option<&Arc<wgpu::ComputePipeline>> {
        self.pipelines.get(key)
    }
    fn contains_resource(&self, key: &CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<wgpu::ComputePipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: CacheKey,
        create_fn: F,
    ) -> Result<Arc<wgpu::ComputePipeline>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::ComputePipeline>, EngineError>,
    {
        self.pipelines.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: CacheKey, resource: Arc<wgpu::ComputePipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &CacheKey) -> Option<Arc<wgpu::ComputePipeline>> {
        self.pipelines.remove(key)
    }
    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<wgpu::ComputePipeline>>
    where
        Arc<wgpu::ComputePipeline>: 'a,
    {
        self.pipelines.values()
    }
}

