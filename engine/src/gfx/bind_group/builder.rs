use wgpu::Device;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource,
    TextureViewDescriptor, TextureViewDimension,
};

use crate::gfx::buffer::WgpuBuffer;
use crate::Texture;

/// Build: camera-only group (matches `global::camera_layout`).
pub fn camera_group(
    device: &Device,
    layout: &BindGroupLayout,
    camera_uniform_buffer: &WgpuBuffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("camera uniform bind group"),
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera_uniform_buffer.get().as_entire_binding(),
        }],
    })
}

/// Build: light-only group (matches `global::light_layout`).
pub fn light_group(
    device: &Device,
    layout: &BindGroupLayout,
    light_uniform_buffer: &WgpuBuffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("light uniform bind group"),
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: light_uniform_buffer.get().as_entire_binding(),
        }],
    })
}

/// Build: camera + light group (matches `global::global_uniform_layout`).
pub fn global_uniform_group(
    device: &Device,
    layout: &BindGroupLayout,
    camera_uniform_buffer: &WgpuBuffer,
    light_uniform_buffer: &WgpuBuffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("global/frame uniform bind group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.get().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: light_uniform_buffer.get().as_entire_binding(),
            },
        ],
    })
}

/// Build: ortho camera group (matches `global::ortho_uniform_layout`).
pub fn ortho_uniform_group(
    device: &Device,
    layout: &BindGroupLayout,
    ortho_uniform_buffer: &WgpuBuffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("ortho uniform bind group"),
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: ortho_uniform_buffer.get().as_entire_binding(),
        }],
    })
}

/// Build: camera + light + debug group (matches `global::debug_layout`).
pub fn debug_group(
    device: &Device,
    layout: &BindGroupLayout,
    camera_uniform_buffer: &WgpuBuffer,
    light_uniform_buffer: &WgpuBuffer,
    debug_uniform_buffer: &WgpuBuffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("debug (camera+light+debug) bind group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.get().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: light_uniform_buffer.get().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: debug_uniform_buffer.get().as_entire_binding(),
            },
        ],
    })
}

/// Build: single 2D texture + sampler (matches `textures::diffuse_layout`).
pub fn diffuse_group(
    device: &Device,
    layout: &BindGroupLayout,
    tex: &Texture,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(&format!("{} diffuse bind group", tex.label)),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&tex.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&tex.sampler),
            },
        ],
    })
}

/// Build: diffuse + normal textures with samplers (matches `textures::normal_texture_layout`).
pub fn normal_textures_group(
    device: &Device,
    layout: &BindGroupLayout,
    diffuse: &Texture,
    normal: &Texture,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(&format!("{} normal-textures bind group", label)),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&diffuse.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&diffuse.sampler),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&normal.view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&normal.sampler),
            },
        ],
    })
}

/// Build: sprite texture array + sampler (matches `textures::sprite_2d_array_layout`).
pub fn sprite_2d_array_group(
    device: &Device,
    layout: &BindGroupLayout,
    texture_array: &Texture,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("sprite 2d array bind group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_array.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&texture_array.sampler),
            },
        ],
    })
}

/// Build: skybox cubemap + sampler (matches `skybox::skybox_cubemap_layout`).
pub fn skybox_cubemap_group(
    device: &Device,
    layout: &BindGroupLayout,
    cubemap: &Texture,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(&format!("{} skybox cubemap bind group", cubemap.label)),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&cubemap.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&cubemap.sampler),
            },
        ],
    })
}

/// Build: equirect→cubemap projection inputs (matches `skybox::skybox_projection_input_layout`).
///
/// `src` is the equirectangular HDR texture; `dst` is the cubemap texture we write into,
/// but we provide its **array view** to the compute shader (D2Array).
pub fn skybox_projection_input_group(
    device: &Device,
    layout: &BindGroupLayout,
    src_equirect: &Texture,
    dst_cubemap: &Texture,
) -> BindGroup {
    let dst_array_view = dst_cubemap.create_view(&TextureViewDescriptor {
        label: Some("Cubemap projection D2Array view"),
        // important: the projection shader expects a 2D array view
        dimension: Some(TextureViewDimension::D2Array),
        ..Default::default()
    });

    device.create_bind_group(&BindGroupDescriptor {
        label: Some(&format!("{} projection input bind group", src_equirect.label)),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&src_equirect.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&dst_array_view),
            },
        ],
    })
}

/// Build: material storage buffer (matches `materials::material_storage_layout`).
pub fn material_storage_group(
    device: &Device,
    layout: &BindGroupLayout,
    material_buffer: &WgpuBuffer,
    label: Option<&str>,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label,
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: material_buffer.get().as_entire_binding(),
        }],
    })
}

/// Convenience alias if you’re binding an HDR 2D env texture like a regular diffuse.
/// (Matches `textures::diffuse_layout`.)
pub fn hdr_group(
    device: &Device,
    layout: &BindGroupLayout,
    hdr_tex: &Texture,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(&format!("{} hdr bind group", label)),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&hdr_tex.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&hdr_tex.sampler),
            },
        ],
    })
}

