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
        bind_group_layouts: Vec<std::sync::Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadModelAsset {
        bind_group_layouts: Vec<std::sync::Arc<wgpu::BindGroupLayout>>,
        asset: crate::ModelAsset,
        format: wgpu::TextureFormat,
    },
    LoadMaterial {
        bind_group_layouts: Vec<std::sync::Arc<wgpu::BindGroupLayout>>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadMaterialAsset {
        bind_group_layouts: Vec<std::sync::Arc<wgpu::BindGroupLayout>>,
        asset: crate::MaterialAsset,
        format: wgpu::TextureFormat,
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
