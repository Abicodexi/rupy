use crate::HashCache;

pub struct ComputePipelineManager {
    pipelines: crate::HashCache<std::sync::Arc<wgpu::ComputePipeline>>,
}
impl ComputePipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: HashCache::new(),
        }
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::ComputePipeline>> for ComputePipelineManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<&mut std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::ComputePipeline>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::ComputePipeline>,
    {
        let start = std::time::Instant::now();

        let pipeline = self.pipelines.entry(key.clone()).or_insert_with(create_fn);
        crate::log_debug!(
            "[PipelineManager] Loaded compute pipeline `{}` in {:.2?}",
            &key.id,
            start.elapsed()
        );
        pipeline
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::ComputePipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.pipelines.remove(key);
    }
}
pub struct RenderPipelineManager {
    pipelines: crate::HashCache<std::sync::Arc<wgpu::RenderPipeline>>,
}
impl RenderPipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: HashCache::new(),
        }
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::RenderPipeline>> for RenderPipelineManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<&mut std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::RenderPipeline>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::RenderPipeline>,
    {
        let start = std::time::Instant::now();
        let pipeline = self.pipelines.entry(key.clone()).or_insert_with(create_fn);
        crate::log_debug!(
            "[PipelineManager] Loaded render pipeline `{}` in {:.2?}",
            &key.id,
            start.elapsed()
        );
        pipeline
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::RenderPipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.pipelines.remove(key);
    }
}
