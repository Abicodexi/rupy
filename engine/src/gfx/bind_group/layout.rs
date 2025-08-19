use std::num::NonZeroU64;
use std::mem::size_of;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, SamplerBindingType, ShaderStages,
    TextureSampleType, TextureViewDimension,
};

/// Declarative entry used to build a BindGroupLayout.
pub struct BindGroupLayoutDefinition {
    pub binding: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
}

fn create_bind_group_layout(
    device: &wgpu::Device,
    label: Option<&str>,
    layout_definitions: &[BindGroupLayoutDefinition],
) -> BindGroupLayout {
    let entries: Vec<BindGroupLayoutEntry> = layout_definitions
        .iter()
        .map(|d| BindGroupLayoutEntry {
            binding: d.binding,
            visibility: d.visibility,
            ty: d.ty.clone(),
            count: None,
        })
        .collect();

    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label,
        entries: &entries,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// GLOBAL / PER-FRAME (camera, lights, debug, ortho)
// ─────────────────────────────────────────────────────────────────────────────

pub mod global {
    use crate::{camera::{Camera, CameraUniform, OrthoUniform}, DebugUniform, Light, LightUniform};

    use super::*;

    /// Camera + Light packed into one per-frame bind group.
    pub fn global_uniform_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0, // Camera
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(
                        size_of::<CameraUniform>() as u64,
                    ).unwrap()),
                },
            },
            BindGroupLayoutDefinition {
                binding: 1, // Light
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(
                        size_of::<LightUniform>() as u64,
                    ).unwrap()),
                },
            },
        ];
        create_bind_group_layout(device, Some("global/frame uniform layout"), defs)
    }

    /// Camera-only layout.
    pub fn camera_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[BindGroupLayoutDefinition {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: Camera::BINDING,
        }];
        create_bind_group_layout(device, Some("camera bind group layout"), defs)
    }

    /// Light-only layout.
    pub fn light_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[BindGroupLayoutDefinition {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: Light::BINDING,
        }];
        create_bind_group_layout(device, Some("light bind group layout"), defs)
    }

    /// Ortho camera uniform for 2D passes / UI.
    pub fn ortho_uniform_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[BindGroupLayoutDefinition {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(NonZeroU64::new(
                    size_of::<OrthoUniform>() as u64,
                ).unwrap()),
            },
        }];
        create_bind_group_layout(device, Some("ortho uniform layout"), defs)
    }

    /// Camera + Light + Debug toggles.
    pub fn debug_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0, // Camera
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(
                        size_of::<CameraUniform>() as u64,
                    ).unwrap()),
                },
            },
            BindGroupLayoutDefinition {
                binding: 1, // Light
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(
                        size_of::<LightUniform>() as u64,
                    ).unwrap()),
                },
            },
            BindGroupLayoutDefinition {
                binding: 2, // Debug
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(
                        size_of::<DebugUniform>() as u64,
                    ).unwrap()),
                },
            },
        ];
        create_bind_group_layout(device, Some("debug bind group layout"), defs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TEXTURES & SAMPLERS (2D, normal, sprite arrays)
// ─────────────────────────────────────────────────────────────────────────────

pub mod textures {
    use crate::Texture;

    use super::*;

    /// Simple 2D texture + filtering sampler (diffuse/baseColor).
    pub fn diffuse_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::TEXTURE_D2_BINDING,
            },
            BindGroupLayoutDefinition {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::SAMPLER_FILTERING_BINDING,
            },
        ];
        create_bind_group_layout(device, Some("diffuse bind group layout"), defs)
    }

    /// Diffuse + Normal textures with samplers.
    pub fn normal_texture_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0, // diffuse
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::TEXTURE_D2_BINDING,
            },
            BindGroupLayoutDefinition {
                binding: 1, // diffuse sampler
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::SAMPLER_FILTERING_BINDING,
            },
            BindGroupLayoutDefinition {
                binding: 2, // normal
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::TEXTURE_D2_BINDING,
            },
            BindGroupLayoutDefinition {
                binding: 3, // normal sampler
                visibility: ShaderStages::FRAGMENT,
                ty: Texture::SAMPLER_FILTERING_BINDING,
            },
        ];
        create_bind_group_layout(device, Some("normal bind group layout"), defs)
    }

    /// Texture-array + sampler (font atlases, sprite sheets).
    pub fn sprite_2d_array_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2Array,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
            },
            BindGroupLayoutDefinition {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
            },
        ];
        create_bind_group_layout(device, Some("sprite 2d array layout"), defs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SKYBOX / ENVIRONMENT MAPS (equirect projection → cubemap)
// ─────────────────────────────────────────────────────────────────────────────

pub mod skybox {
    use crate::Texture;

    use super::*;

    /// Input textures for equirect→cubemap projection (compute stage).
    pub fn skybox_projection_input_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: Texture::PROJECTION[0],
            },
            BindGroupLayoutDefinition {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: Texture::PROJECTION[1],
            },
        ];
        create_bind_group_layout(device, Some("skybox projection input layout"), defs)
    }

    /// Cubemap + sampler used when sampling the skybox.
    pub fn skybox_cubemap_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[
            BindGroupLayoutDefinition {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::Cube,
                    multisampled: false,
                },
            },
            BindGroupLayoutDefinition {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
            },
        ];
        create_bind_group_layout(device, Some("skybox cubemap layout"), defs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MATERIALS & DATA BUFFERS
// ─────────────────────────────────────────────────────────────────────────────

pub mod materials {
    use super::*;

    /// Read-only storage buffer for material data (array-friendly).
    pub fn material_storage_layout(device: &wgpu::Device) -> BindGroupLayout {
        let defs = &[BindGroupLayoutDefinition {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                // None → allow arbitrary-sized arrays in the SSBO.
                min_binding_size: None,
            },
        }];
        create_bind_group_layout(device, Some("material storage layout"), defs)
    }
}

pub use global::{
    camera_layout,
    debug_layout,
    global_uniform_layout,
    light_layout,
    ortho_uniform_layout,
};

pub use textures::{
    diffuse_layout,
    normal_texture_layout,
    sprite_2d_array_layout,
};

pub use skybox::{
    skybox_cubemap_layout,
    skybox_projection_input_layout,
};

pub use materials::material_storage_layout;

