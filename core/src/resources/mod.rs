pub mod bind_group;
pub use bind_group::*;

pub mod buffer;
pub use buffer::*;

pub mod cache;
pub use cache::*;

pub mod cache_key;
pub use cache_key::*;

pub mod texture;
pub use texture::*;

pub struct Managers {
    pub queue: std::sync::Arc<wgpu::Queue>,
    pub device: std::sync::Arc<wgpu::Device>,
    pub shader_manager: crate::ShaderManager,
    pub compute_pipeline_manager: crate::ComputePipelineManager,
    pub render_pipeline_manager: crate::RenderPipelineManager,
    pub buffer_manager: BufferManager,
    pub texture_manager: TextureManager,
    pub mesh_manager: crate::MeshManager,
    pub material_manager: crate::MaterialManager,
    pub model_manager: crate::ModelManager,
    pub bind_group_manager: crate::BindGroupManager,
}

impl Managers {
    pub fn new(queue: &std::sync::Arc<wgpu::Queue>, device: &std::sync::Arc<wgpu::Device>) -> Self {
        let shader_manager = crate::ShaderManager::new();
        let texture_manager = TextureManager::new();
        let render_pipeline_manager = crate::RenderPipelineManager::new();
        let compute_pipeline_manager = crate::ComputePipelineManager::new();
        let buffer_manager = BufferManager::new();
        let mesh_manager = crate::MeshManager::new();
        let material_manager = crate::MaterialManager::new();
        let model_manager = crate::ModelManager::new();
        let bind_group_manager = BindGroupManager::new();
        Managers {
            queue: queue.clone(),
            device: device.clone(),
            shader_manager,
            render_pipeline_manager,
            compute_pipeline_manager,
            buffer_manager,
            texture_manager,
            mesh_manager,
            material_manager,
            model_manager,
            bind_group_manager,
        }
    }
}
