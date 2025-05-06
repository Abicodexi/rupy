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
