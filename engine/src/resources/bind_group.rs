use std::sync::Arc;

use crate::{
    camera::{Camera, CameraUniform, OrthoUniform},
    log_error, CacheKey, DebugUniform, EngineError, Light, LightUniform, Texture, WgpuBuffer,
};

use super::{CacheStorage, HashCache, TextureManager};

pub struct BindingDef {
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
}

fn create_layout(
    device: &wgpu::Device,
    label: Option<&str>,
    defs: &[BindingDef],
) -> wgpu::BindGroupLayout {
    let entries: Vec<wgpu::BindGroupLayoutEntry> = defs
        .iter()
        .map(|d| wgpu::BindGroupLayoutEntry {
            binding: d.binding,
            visibility: d.visibility,
            ty: d.ty.clone(),
            count: None,
        })
        .collect();

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &entries,
    })
}

pub struct RenderBindGroupLayouts {
    diffuse: Arc<wgpu::BindGroupLayout>,
    light: Arc<wgpu::BindGroupLayout>,
    camera: Arc<wgpu::BindGroupLayout>,
    equirect_src: Arc<wgpu::BindGroupLayout>,
    equirect_dst: Arc<wgpu::BindGroupLayout>,
    uniform: Arc<wgpu::BindGroupLayout>,
    normal: Arc<wgpu::BindGroupLayout>,
    material_storage: Arc<wgpu::BindGroupLayout>,
    debug: Arc<wgpu::BindGroupLayout>,
    sprite_2d_array: Arc<wgpu::BindGroupLayout>,
    ortho_uniform: Arc<wgpu::BindGroupLayout>,
}

impl RenderBindGroupLayouts {
    pub fn material_storage(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.material_storage
    }
    pub fn texture(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.diffuse
    }
    pub fn light(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.light
    }
    pub fn camera(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.camera
    }
    pub fn equirect_src(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.equirect_src
    }
    pub fn equirect_dst(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.equirect_dst
    }
    pub fn uniform(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.uniform
    }
    pub fn normal(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.normal
    }
    pub fn debug(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.debug
    }
    pub fn sprite_2d_array(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.sprite_2d_array
    }
    pub fn ortho_uniform(&self) -> &Arc<wgpu::BindGroupLayout> {
        &self.ortho_uniform
    }

    pub fn new(device: &wgpu::Device) -> Self {
        // Diffuse textures (2D)
        let diffuse_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: Texture::TEXTURE_D2_BINDING,
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: Texture::SAMPLER_FILTERING_BINDING,
            },
        ];
        let diffuse = create_layout(&device, Some("texture bind group layout"), diffuse_defs);

        // Light uniform (single buffer)
        let light_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: Light::BINDING,
        }];
        let light = create_layout(&device, Some("light bind group layout"), light_defs);

        // Camera uniform (single buffer)
        let camera_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: Camera::BINDING,
        }];
        let camera = create_layout(&device, Some("camera bind group layout"), camera_defs);

        // Equirectangular (dual texture)
        let equirect_src_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: Texture::PROJECTION[0],
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: Texture::PROJECTION[1],
            },
        ];
        let equirect_src = create_layout(&device, Some("equirect src layout"), equirect_src_defs);

        // Equirectangular (texture + sampler)
        let equirect_dst_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            },
        ];
        let equirect_dst = create_layout(&device, Some("equirect dst layout"), equirect_dst_defs);

        // Combined uniform (camera + light)
        let uniform_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<CameraUniform>() as u64,
                    ),
                },
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<LightUniform>() as u64,
                    ),
                },
            },
        ];
        let uniform = create_layout(&device, Some("uniform bind group layout"), uniform_defs);

        // Normal maps (RGBA textures + sampler)
        let normal_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::TEXTURE_D2_BINDING,
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::SAMPLER_FILTERING_BINDING,
            },
            BindingDef {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::TEXTURE_D2_BINDING,
            },
            BindingDef {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::SAMPLER_FILTERING_BINDING,
            },
        ];
        let normal = create_layout(&device, Some("normal bind group layout"), normal_defs);

        let material_storage_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(
                    std::mem::size_of::<crate::MaterialData>() as u64,
                ),
            },
        }];
        let material_storage = create_layout(
            &device,
            Some("material storage bind group layout"),
            material_storage_defs,
        );

        // Combined uniform (camera + light)
        let debug_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<CameraUniform>() as u64,
                    ),
                },
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<LightUniform>() as u64,
                    ),
                },
            },
            BindingDef {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<DebugUniform>() as u64)
                            .unwrap(),
                    ),
                },
            },
        ];
        let debug = create_layout(&device, Some("debug bind grop layout"), debug_defs);

        let sprite_2d_array_defs = &[
            // binding 0 → a 2D texture‐array (one layer per font page)
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            },
            // binding 1 → a filtering sampler
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            },
        ];
        let sprite_2d_array = create_layout(
            &device,
            Some("sprite_fonts bind group layout"),
            sprite_2d_array_defs,
        );
        let ortho_uniform_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(
                    std::mem::size_of::<OrthoUniform>() as u64
                ),
            },
        }];
        let ortho_uniform = create_layout(
            &device,
            Some("ortho uniform bind group layout"),
            ortho_uniform_defs,
        );
        RenderBindGroupLayouts {
            diffuse: diffuse.into(),
            light: light.into(),
            camera: camera.into(),
            equirect_src: equirect_src.into(),
            equirect_dst: equirect_dst.into(),
            uniform: uniform.into(),
            normal: normal.into(),
            material_storage: material_storage.into(),
            debug: debug.into(),
            sprite_2d_array: sprite_2d_array.into(),
            ortho_uniform: ortho_uniform.into(),
        }
    }
}

pub struct BindGroup;

impl BindGroup {
    pub fn ortho_uniform(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        ortho_uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layouts.ortho_uniform(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ortho_uniform_buffer.get().as_entire_binding(),
            }],
            label: Some("combined UBO bind group"),
        })
    }
    pub fn equirect_dst(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        dst: &super::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} projection destination bind group", dst.label)),
            layout: bind_group_layouts.equirect_dst(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dst.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dst.sampler),
                },
            ],
        })
    }
    pub fn equirect_src(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        src: &super::Texture,
        dst: &super::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layouts.equirect_src(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: Some("Cubemap projection view"),
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            ..Default::default()
                        },
                    )),
                },
            ],
            label: Some(&format!("{}  projection source bind group", src.label)),
        })
    }

    pub fn camera(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera uniform bind group"),
            layout: bind_group_layouts.camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn light(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light uniform bind group"),
            layout: bind_group_layouts.light(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn uniform(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        camera_uniform_buffer: &crate::WgpuBuffer,
        light_uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layouts.uniform(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.get().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_uniform_buffer.get().as_entire_binding(),
                },
            ],
            label: Some("combined UBO bind group"),
        })
    }
    pub fn texture(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        diffuse: &super::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group", diffuse.label)),
            layout: bind_group_layouts.texture(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse.sampler),
                },
            ],
        })
    }

    pub fn normal(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        diffuse: &std::sync::Arc<super::Texture>,
        normal: &std::sync::Arc<super::Texture>,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group layout", label)),
            layout: bind_group_layouts.normal(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal.sampler),
                },
            ],
        })
    }
    pub fn hdr(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        hdr: &super::Texture,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group layout", label)),
            layout: bind_group_layouts.texture(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&hdr.sampler),
                },
            ],
        })
    }
    pub fn material_storage(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        material_buffer: &WgpuBuffer,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layouts.material_storage(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn debug(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        camera_uniform_buffer: &WgpuBuffer,
        light_uniform_buffer: &WgpuBuffer,
        debug: &WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layouts.debug(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.get().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_uniform_buffer.get().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: debug.get().as_entire_binding(),
                },
            ],
            label: Some("combined UBO+debug bind group"),
        })
    }
    pub fn sprite_2d_array(
        device: &wgpu::Device,
        bind_group_layouts: &RenderBindGroupLayouts,
        texture: &super::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("sprite 2d bind group")),
            layout: bind_group_layouts.sprite_2d_array(),
            entries: &[
                // binding 0 → texture array view
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                // binding 1 → sampler
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}

pub struct BindGroupManager {
    bind_groups: HashCache<std::sync::Arc<wgpu::BindGroup>>,
}

impl BindGroupManager {
    pub fn new() -> Self {
        Self {
            bind_groups: HashCache::new(),
        }
    }
    pub fn bind_group_for(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        textures: &mut TextureManager,
        texture: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        let texture_key = CacheKey::from(texture);
        if let Some(bind_group) = self.bind_groups.get(&texture_key) {
            return Some(bind_group.clone());
        }
        if !textures.contains_resource(&texture_key) {
            if let Err(e) = textures.load(queue, device, texture) {
                log_error!(
                    "Failed to get bind group for {}, error loading texture: {}",
                    texture,
                    e.to_string()
                );
            }
        }
        if let Some(tex) = textures.get_resource(&texture_key) {
            let bind_group_entries = [
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
            ];
            let bind_group: std::sync::Arc<wgpu::BindGroup> = device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("{} bind group", texture)),
                    layout,
                    entries: &bind_group_entries,
                })
                .into();
            self.bind_groups
                .insert(texture_key.clone(), bind_group.clone());
        }
        self.bind_groups.get(&texture_key).cloned()
    }
}

impl CacheStorage<std::sync::Arc<wgpu::BindGroup>> for BindGroupManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.get(key)
    }

    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.bind_groups.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<wgpu::BindGroup>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::BindGroup>, EngineError>,
    {
        self.bind_groups.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::BindGroup>) {
        self.bind_groups.insert(key, resource);
    }
    fn remove_resource(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<wgpu::BindGroup>>
    where
        std::sync::Arc<wgpu::BindGroup>: 'a,
    {
        self.bind_groups.values()
    }
}
