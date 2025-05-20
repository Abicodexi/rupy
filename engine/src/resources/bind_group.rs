use crate::DebugUniform;

use super::{CacheStorage, HashCache, TextureManager};

pub struct BindGroupBindingType {
    pub(crate) binding: wgpu::BindingType,
}

/// A single binding definition.
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

/// Holds all bind-group layouts.
pub struct RenderBindGroupLayouts {
    pub device: std::sync::Arc<wgpu::Device>,
    pub diffuse: wgpu::BindGroupLayout,
    pub light: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub equirect_src: wgpu::BindGroupLayout,
    pub equirect_dst: wgpu::BindGroupLayout,
    pub uniform: wgpu::BindGroupLayout,
    pub normal: wgpu::BindGroupLayout,
    pub material_storage: wgpu::BindGroupLayout,
    pub debug: wgpu::BindGroupLayout,
}

impl RenderBindGroupLayouts {
    /// Initialize and return the singleton.
    pub fn get() -> &'static Self {
        static LAYOUTS: once_cell::sync::OnceCell<RenderBindGroupLayouts> =
            once_cell::sync::OnceCell::new();
        LAYOUTS.get_or_init(|| {
            let binding = crate::GPU::get();
            let gpu = binding.read().expect("GPU resources not initialized");
            RenderBindGroupLayouts::new(gpu.device().clone())
        })
    }
    pub fn material_storage() -> &'static wgpu::BindGroupLayout {
        &Self::get().material_storage
    }
    pub fn texture() -> &'static wgpu::BindGroupLayout {
        &Self::get().diffuse
    }
    pub fn light() -> &'static wgpu::BindGroupLayout {
        &Self::get().light
    }
    pub fn camera() -> &'static wgpu::BindGroupLayout {
        &Self::get().camera
    }
    pub fn equirect_src() -> &'static wgpu::BindGroupLayout {
        &Self::get().equirect_src
    }
    pub fn equirect_dst() -> &'static wgpu::BindGroupLayout {
        &Self::get().equirect_dst
    }
    pub fn uniform() -> &'static wgpu::BindGroupLayout {
        &Self::get().uniform
    }
    pub fn normal() -> &'static wgpu::BindGroupLayout {
        &Self::get().normal
    }
    pub fn debug() -> &'static wgpu::BindGroupLayout {
        &Self::get().debug
    }

    fn new(device: std::sync::Arc<wgpu::Device>) -> Self {
        // Diffuse textures (2D)
        let diffuse_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::D2[0].binding.clone(),
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::D2[1].binding.clone(),
            },
        ];
        let diffuse = create_layout(&device, Some("texture bind group layout"), diffuse_defs);

        // Light uniform (single buffer)
        let light_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: crate::Light::BUFFER_BINDING.binding.clone(),
        }];
        let light = create_layout(&device, Some("light bind group layout"), light_defs);

        // Camera uniform (single buffer)
        let camera_defs = &[BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: crate::camera::Camera::UNIFORM_BUFFER_BINDING
                .binding
                .clone(),
        }];
        let camera = create_layout(&device, Some("camera bind group layout"), camera_defs);

        // Equirectangular (dual texture)
        let equirect_src_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: crate::Texture::PROJECTION[0].binding.clone(),
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: crate::Texture::PROJECTION[1].binding.clone(),
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
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::camera::uniform::CameraUniform,
                    >() as u64),
                },
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::light::LightUniform,
                    >() as u64),
                },
            },
        ];
        let uniform = create_layout(&device, Some("uniform bind group layout"), uniform_defs);

        // Normal maps (RGBA textures + sampler)
        let normal_defs = &[
            BindingDef {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::NORMAL[0].binding.clone(),
            },
            BindingDef {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::NORMAL[1].binding.clone(),
            },
            BindingDef {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::NORMAL[2].binding.clone(),
            },
            BindingDef {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: crate::Texture::NORMAL[3].binding.clone(),
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

        let debug_defs = [BindingDef {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(
                    std::num::NonZeroU64::new(std::mem::size_of::<DebugUniform>() as u64).unwrap(),
                ),
            },
        }];
        let debug = create_layout(&device, Some("debug bind grop layout"), &debug_defs);

        RenderBindGroupLayouts {
            device: device.clone(),
            diffuse,
            light,
            camera,
            equirect_src,
            equirect_dst,
            uniform,
            normal,
            material_storage,
            debug,
        }
    }
}

pub struct BindGroup;

impl BindGroup {
    pub fn equirect_dst(device: &wgpu::Device, dst: &super::Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} projection destination bind group", dst.label)),
            layout: RenderBindGroupLayouts::equirect_dst(),
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
            layout: crate::RenderBindGroupLayouts::equirect_src(),
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
            layout: &crate::RenderBindGroupLayouts::camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn light(device: &wgpu::Device, uniform_buffer: &crate::WgpuBuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light uniform bind group"),
            layout: &crate::RenderBindGroupLayouts::camera(),
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
            layout: &crate::RenderBindGroupLayouts::uniform(),
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
            layout: RenderBindGroupLayouts::texture(),
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
            layout: RenderBindGroupLayouts::normal(),
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
            layout: RenderBindGroupLayouts::texture(),
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
        material_buffer: &crate::WgpuBuffer,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: crate::RenderBindGroupLayouts::material_storage(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.get().as_entire_binding(),
            }],
        })
    }
}

pub struct BindGroupManager {
    bind_groups: super::HashCache<std::sync::Arc<wgpu::BindGroup>>,
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
        key: &super::CacheKey,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        let binding = crate::GPU::get();
        if let Ok(gpu) = binding.read() {
            if !self.bind_groups.contains(&key) {
                let tex = texture_manager.get(*key)?;
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
