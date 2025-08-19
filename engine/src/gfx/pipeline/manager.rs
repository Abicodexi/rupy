use super::{ComputePipelineManager, RenderPipelineManager};

/// Top-level manager that holds both compute + render pipeline caches.
pub struct PipelineManager {
    pub render: RenderPipelineManager,
    pub compute: ComputePipelineManager,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            render: RenderPipelineManager::new(),
            compute: ComputePipelineManager::new(),
        }
    }
}

