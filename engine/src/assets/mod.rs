pub mod loader;
pub use loader::*;

pub mod watcher;
pub use watcher::*;

pub mod service;
pub use service::*;
#[derive(Debug, Clone)]

pub enum AssetRequest {
    Shutdown,
    LoadShader {
        shader: String,
    },
    LoadTexture {
        texture: String,
    },
    LoadModel {
        file: String,
        v_shader: String,
        f_shader: String,
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadModelAsset {
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        asset: crate::ModelAsset,
    },
    LoadMaterial {
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadMaterialAsset {
        bind_group_layouts: Vec<wgpu::BindGroupLayout>,
        asset: crate::MaterialAsset,
    },
    LoadRenderPipeline {
        layout: wgpu::PipelineLayout,
        f_shader: String,
        v_shader: String,
        buffers: Vec<crate::OwnedVertexBufferLayout>,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
        key: crate::CacheKey,
        label: String,
    },
}
