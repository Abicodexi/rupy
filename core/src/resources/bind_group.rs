use super::{CacheStorage, HashCache, TextureManager};

pub struct BindGroupBindingType {
    pub(crate) binding: wgpu::BindingType,
}

/// A single binding definition\, used to build layouts from data-driven descriptors.
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

/// Holds all of your shared bind-group layouts, initialized once per device.
pub struct BindGroupLayouts {
    pub device: std::sync::Arc<wgpu::Device>,
    pub diffuse: wgpu::BindGroupLayout,
    pub light: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub equirect_src: wgpu::BindGroupLayout,
    pub equirect_dst: wgpu::BindGroupLayout,
    pub uniform: wgpu::BindGroupLayout,
    pub normal: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    /// Lazily initialize and return the singleton for this device.
    pub fn get() -> &'static Self {
        static LAYOUTS: once_cell::sync::OnceCell<BindGroupLayouts> =
            once_cell::sync::OnceCell::new();
        LAYOUTS.get_or_init(|| {
            let binding = crate::GPU::get();
            let gpu = binding.read().expect("GPU resources not initialized");
            BindGroupLayouts::new(gpu.device().clone())
        })
    }

    /// Accessors for each layout.
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

        // Equirectangular compute src
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

        // Equirectangular render dst
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

        BindGroupLayouts {
            device: device.clone(),
            diffuse,
            light,
            camera,
            equirect_src,
            equirect_dst,
            uniform,
            normal,
        }
    }
}

pub struct BindGroup;

impl BindGroup {
    pub fn equirect_dst(device: &wgpu::Device, dst: &super::Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} projection destination bind group", dst.label)),
            layout: BindGroupLayouts::equirect_dst(),
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
            layout: crate::BindGroupLayouts::equirect_src(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("Cubemap projection view"),
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    })),
                },
            ],
            label: Some(&format!("{}  projection source bind group", src.label,)),
        })
    }

    pub fn camera(device: &wgpu::Device, uniform_buffer: &crate::WgpuBuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera uniform bind group"),
            layout: &crate::BindGroupLayouts::camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        })
    }
    pub fn light(device: &wgpu::Device, uniform_buffer: &crate::WgpuBuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light uniform bind group"),
            layout: &crate::BindGroupLayouts::camera(),
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
            layout: &crate::BindGroupLayouts::uniform(),
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
            layout: BindGroupLayouts::texture(),
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
            layout: BindGroupLayouts::normal(),
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
            layout: BindGroupLayouts::texture(),
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
    fn remove(&mut self, key: &crate::CacheKey) {
        self.bind_groups.remove(key);
    }
}
