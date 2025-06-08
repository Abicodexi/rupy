use std::sync::Arc;

use crate::{
    camera::{Camera, CameraUniform, OrthoUniform},
    CacheKey, DebugUniform, Light, LightUniform, Texture, WgpuBuffer,
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
    pub device: std::sync::Arc<wgpu::Device>,
    pub diffuse: Arc<wgpu::BindGroupLayout>,
    pub light: Arc<wgpu::BindGroupLayout>,
    pub camera: Arc<wgpu::BindGroupLayout>,
    pub equirect_src: Arc<wgpu::BindGroupLayout>,
    pub equirect_dst: Arc<wgpu::BindGroupLayout>,
    pub uniform: Arc<wgpu::BindGroupLayout>,
    pub normal: Arc<wgpu::BindGroupLayout>,
    pub material_storage: Arc<wgpu::BindGroupLayout>,
    pub debug: Arc<wgpu::BindGroupLayout>,
    pub sprite_2d: Arc<wgpu::BindGroupLayout>,
    pub ortho_uniform: Arc<wgpu::BindGroupLayout>,
}
static LAYOUTS: once_cell::sync::OnceCell<Arc<RenderBindGroupLayouts>> =
    once_cell::sync::OnceCell::new();

impl RenderBindGroupLayouts {
    pub fn get<'a>() -> &'a Arc<RenderBindGroupLayouts> {
        if LAYOUTS.get().is_none() {
            RenderBindGroupLayouts::init();
        }
        LAYOUTS
            .get()
            .expect("Static layouts must exist at this point")
    }
    pub fn init() {
        let binding = crate::GPU::get();
        let gpu = binding.read().expect("GPU resources not initialized");
        let layouts = RenderBindGroupLayouts::new(gpu.device().clone());
        LAYOUTS.set(layouts.into()).ok();
    }
    pub fn material_storage() -> Arc<wgpu::BindGroupLayout> {
        Self::get().material_storage.clone()
    }
    pub fn texture() -> Arc<wgpu::BindGroupLayout> {
        Self::get().diffuse.clone()
    }
    pub fn light() -> Arc<wgpu::BindGroupLayout> {
        Self::get().light.clone()
    }
    pub fn camera() -> Arc<wgpu::BindGroupLayout> {
        Self::get().camera.clone()
    }
    pub fn equirect_src() -> Arc<wgpu::BindGroupLayout> {
        Self::get().equirect_src.clone()
    }
    pub fn equirect_dst() -> Arc<wgpu::BindGroupLayout> {
        Self::get().equirect_dst.clone()
    }
    pub fn uniform() -> Arc<wgpu::BindGroupLayout> {
        Self::get().uniform.clone()
    }
    pub fn normal() -> Arc<wgpu::BindGroupLayout> {
        Self::get().normal.clone()
    }
    pub fn debug() -> Arc<wgpu::BindGroupLayout> {
        Self::get().debug.clone()
    }
    pub fn sprite_2d() -> Arc<wgpu::BindGroupLayout> {
        Self::get().sprite_2d.clone()
    }
    pub fn ortho_uniform() -> Arc<wgpu::BindGroupLayout> {
        Self::get().ortho_uniform.clone()
    }

    fn new(device: std::sync::Arc<wgpu::Device>) -> Self {
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

        let sprite_2d_defs = &[
            // binding 0 → a 2D texture‐array (one layer per font page)
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
            },
            // binding 1 → a filtering sampler
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            },
        ];
        let sprite_2d = create_layout(
            &device,
            Some("sprite_fonts bind group layout"),
            sprite_2d_defs,
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
            device: device.clone(),
            diffuse: diffuse.into(),
            light: light.into(),
            camera: camera.into(),
            equirect_src: equirect_src.into(),
            equirect_dst: equirect_dst.into(),
            uniform: uniform.into(),
            normal: normal.into(),
            material_storage: material_storage.into(),
            debug: debug.into(),
            sprite_2d: sprite_2d.into(),
            ortho_uniform: ortho_uniform.into(),
        }
    }
}

pub struct BindGroup;

impl BindGroup {
    pub fn ortho_uniform(
        device: &wgpu::Device,
        ortho_uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &RenderBindGroupLayouts::ortho_uniform(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ortho_uniform_buffer.get().as_entire_binding(),
            }],
            label: Some("combined UBO bind group"),
        })
    }
    pub fn equirect_dst(device: &wgpu::Device, dst: &super::Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} projection destination bind group", dst.label)),
            layout: RenderBindGroupLayouts::equirect_dst().as_ref(),
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
        src: &super::Texture,
        dst: &super::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: RenderBindGroupLayouts::equirect_src().as_ref(),
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

    pub fn camera(device: &wgpu::Device, uniform_buffer: &crate::WgpuBuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera uniform bind group"),
            layout: RenderBindGroupLayouts::camera().as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn light(device: &wgpu::Device, uniform_buffer: &crate::WgpuBuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light uniform bind group"),
            layout: RenderBindGroupLayouts::light().as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn uniform(
        device: &wgpu::Device,
        camera_uniform_buffer: &crate::WgpuBuffer,
        light_uniform_buffer: &crate::WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: RenderBindGroupLayouts::uniform().as_ref(),
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
    pub fn texture(device: &wgpu::Device, diffuse: &super::Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group", diffuse.label)),
            layout: RenderBindGroupLayouts::texture().as_ref(),
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
        diffuse: &std::sync::Arc<super::Texture>,
        normal: &std::sync::Arc<super::Texture>,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group layout", label)),
            layout: RenderBindGroupLayouts::normal().as_ref(),
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
    pub fn hdr(device: &wgpu::Device, hdr: &super::Texture, label: &str) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group layout", label)),
            layout: RenderBindGroupLayouts::texture().as_ref(),
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
        material_buffer: &WgpuBuffer,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: RenderBindGroupLayouts::material_storage().as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn debug(
        device: &wgpu::Device,
        camera_uniform_buffer: &WgpuBuffer,
        light_uniform_buffer: &WgpuBuffer,
        debug: &WgpuBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: RenderBindGroupLayouts::debug().as_ref(),
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
    pub fn sprite2d(device: &wgpu::Device, texture: &super::Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("sprite 2d bind group")),
            layout: RenderBindGroupLayouts::sprite_2d().as_ref(),
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
    pub fn bind_group(&self, key: &super::CacheKey) -> Option<&std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.get(key)
    }
    pub fn bind_group_for(
        &mut self,
        texture_manager: &TextureManager,
        key: &CacheKey,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        let binding = crate::GPU::get();
        if let Ok(gpu) = binding.read() {
            if !self.bind_groups.contains(&key) {
                let tex = texture_manager.get(key)?;
                let bind_group: std::sync::Arc<wgpu::BindGroup> = gpu
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("tex_bg:{}", key.id())),
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&tex.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&tex.sampler),
                            },
                        ],
                    })
                    .into();
                self.bind_groups.insert(key.clone(), bind_group);
            }
        }

        self.bind_groups.get(key).cloned()
    }
}

impl super::CacheStorage<std::sync::Arc<wgpu::BindGroup>> for BindGroupManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.bind_groups.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::BindGroup>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::BindGroup>,
    {
        self.bind_groups.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::BindGroup>) {
        self.bind_groups.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups.remove(key)
    }
}
