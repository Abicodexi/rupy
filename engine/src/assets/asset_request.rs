use std::sync::Arc;

use crate::{CacheKey, MaterialAsset, ModelAsset};

pub enum AssetRequest {
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
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadModelAsset {
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        asset: ModelAsset,
        format: wgpu::TextureFormat,
    },
    LoadMaterial {
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::ColorTargetState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    },
    LoadMaterialAsset {
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        asset: MaterialAsset,
        format: wgpu::TextureFormat,
    },
    LoadRenderPipeline {
        layout: wgpu::PipelineLayout,
        f_shader: String,
        v_shader: String,
        format: wgpu::TextureFormat,
        key: CacheKey,
        label: String,
    },
}
