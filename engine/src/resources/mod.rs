pub mod material;
pub use material::*;

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

pub struct LoadObjContext<'a> {
    pub queue: &'a wgpu::Queue,
    pub device: &'a wgpu::Device,
    pub model_manager: &'a mut ModelManager,
    pub material_manager: &'a mut MaterialManager,
    pub texture_manager: &'a mut TextureManager,
    pub shader_manager: &'a mut crate::ShaderManager,
    pub pipeline_manager: &'a mut crate::gfx::pipeline::PipelineManager,
    pub bind_group_manager: &'a mut crate::gfx::bind_group::BindGroupManager,
}

pub struct ObjectDescriptor<'a> {
    pub file: &'a str,
    pub v_shader: &'a str,
    pub f_shader: &'a str,
    pub buffers: &'a [wgpu::VertexBufferLayout<'a>],
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub primitive: wgpu::PrimitiveState,
    pub format: wgpu::TextureFormat,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
}
