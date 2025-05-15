pub mod bind_group;
pub use bind_group::*;

pub mod material;
pub use material::*;

pub mod buffer;
pub use buffer::*;

pub mod cache;
pub use cache::*;

pub mod cache_key;
pub use cache_key::*;

pub mod texture;
pub use texture::*;

pub mod mesh;
pub use mesh::*;

pub mod model;
pub use model::*;

pub struct Managers {
    pub queue: std::sync::Arc<wgpu::Queue>,
    pub device: std::sync::Arc<wgpu::Device>,
    pub shader_manager: crate::ShaderManager,
    pub pipeline_manager: crate::PipelineManager,
    pub buffer_manager: BufferManager,
    pub texture_manager: TextureManager,
    pub material_manager: crate::MaterialManager,
    pub bind_group_manager: crate::BindGroupManager,
}

impl Managers {
    fn new(queue: std::sync::Arc<wgpu::Queue>, device: std::sync::Arc<wgpu::Device>) -> Self {
        let shader_manager = crate::ShaderManager::new();
        let texture_manager = TextureManager::new();
        let buffer_manager = BufferManager::new();
        let material_manager = crate::MaterialManager::new(&device);
        let bind_group_manager = BindGroupManager::new();
        let pipeline_manager = crate::PipelineManager::new();
        Managers {
            queue: queue.clone(),
            device: device.clone(),
            shader_manager,
            pipeline_manager,
            buffer_manager,
            texture_manager,
            material_manager,
            bind_group_manager,
        }
    }
}

impl Into<Managers> for (&std::sync::Arc<wgpu::Queue>, &std::sync::Arc<wgpu::Device>) {
    fn into(self) -> Managers {
        Managers::new(self.0.clone(), self.1.clone())
    }
}
impl Into<Managers> for (std::sync::Arc<wgpu::Queue>, std::sync::Arc<wgpu::Device>) {
    fn into(self) -> Managers {
        Managers::new(self.0.clone(), self.1.clone())
    }
}

impl Into<Managers> for std::sync::RwLockReadGuard<'_, crate::GPU> {
    fn into(self) -> Managers {
        Managers::new(self.queue().clone(), self.device().clone())
    }
}
