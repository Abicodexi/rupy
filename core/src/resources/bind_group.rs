use super::{CacheStorage, HashCache, TextureManager};

static BGL: once_cell::sync::OnceCell<BindGroupLayouts> = once_cell::sync::OnceCell::new();

fn init_bind_group_layouts(device: &wgpu::Device) {
    BGL.get_or_init(|| BindGroupLayouts::new(device));
}

fn diffuse_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").texture
}
fn uniform_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").uniform
}

fn camera_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").camera
}
fn normal_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").normal
}
fn equirect_src_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").equirect_src
}
fn equirect_dst_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").equirect_dst
}
fn light_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").light
}

pub struct BindGroupEntry{
    stages: wgpu::ShaderStages,    
    binding_type: wgpu::BindingType,

};
pub struct BindGroupBinding {
    bindings: Vec<BindGroupEntry>

};

impl BindGroupBinding {
    pub const TEXTURE2D: BindGroupBinding = BindGroupBinding {
        bindings: vec![
            BindGroupEntry {
                stages: wgpu::ShaderStages::FRAGMENT,
                binding_type:  wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                }

            },
            BindGroupEntry {
                stages: wgpu::ShaderStages::FRAGMENT,
                binding_type                 wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),

            }
        ]
    }
}


pub struct BindGroupLayoutBuilder<'a> {
    device: &'a wgpu::Device,
    label: Option<&'a str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            label: None,
            entries: Vec::new(),
        }
    }
    pub fn label(mut self, lbl: &'a str) -> Self {
        self.label = Some(lbl);
        self
    }
    pub fn binding(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BindingType,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        });
        self
    }
    pub fn build(self) -> wgpu::BindGroupLayout {
        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &self.entries,
            })
    }
}

pub struct BindGroupLayouts {
    pub diffuse: wgpu::BindGroupLayout,
    pub light: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub equirect_src: wgpu::BindGroupLayout,
    pub equirect_dst: wgpu::BindGroupLayout,
    pub uniform: wgpu::BindGroupLayout,
    pub normal: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    pub fn init(device: &wgpu::Device) {
        init_bind_group_layouts(device);
    }
    pub fn uniform() -> &'static wgpu::BindGroupLayout {
        uniform_bind_group_layout()
    }
    pub fn normal() -> &'static wgpu::BindGroupLayout {
        normal_bind_group_layout()
    }
    pub fn camera() -> &'static wgpu::BindGroupLayout {
        camera_bind_group_layout()
    }
    pub fn light() -> &'static wgpu::BindGroupLayout {
        light_bind_group_layout()
    }

    pub fn diffuse() -> &'static wgpu::BindGroupLayout {
        diffuse_bind_group_layout()
    }
    pub fn equirect_src() -> &'static wgpu::BindGroupLayout {
        equirect_src_bind_group_layout()
    }
    pub fn equirect_dst() -> &'static wgpu::BindGroupLayout {
        equirect_dst_bind_group_layout()
    }
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = BindGroupLayoutBuilder::new(&device)
            .label("texture bind group layout")
            .binding(
                0,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            )
            .binding(
                1,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .build();
        let light = BindGroupLayoutBuilder::new(&device)
            .label("texture bind group layout")
            .binding(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::LightUniform,
                    >() as u64),
                },
            )
            .build();

        let camera = BindGroupLayoutBuilder::new(&device)
            .label("camera bind group layout")
            .binding(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::camera::uniform::CameraUniform,
                    >() as u64),
                },
            )
            .build();

        let equirect_src = BindGroupLayoutBuilder::new(&device)
            .binding(
                0,
                wgpu::ShaderStages::COMPUTE,
                wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
            )
            .binding(
                1,
                wgpu::ShaderStages::COMPUTE,
                wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
            )
            .build();

        let equirect_dst = BindGroupLayoutBuilder::new(&device)
            .binding(
                0,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
            )
            .binding(
                1,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            )
            .build();

        let uniform = BindGroupLayoutBuilder::new(&device)
            .binding(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::camera::uniform::CameraUniform,
                    >() as _),
                },
            )
            .binding(
                1,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        crate::light::LightUniform,
                    >() as _),
                },
            )
            .build();

        let normal = BindGroupLayoutBuilder::new(&device)
            .binding(
                0,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            )
            .binding(
                1,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .binding(
                2,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            )
            .binding(
                3,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .build();

        Self {
            texture,
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
    pub fn normal_map(
        device: &wgpu::Device,
        diffuse: &std::sync::Arc<super::Texture>,
        normal: &std::sync::Arc<super::Texture>,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} texture bind group layout", label)),
            layout: BindGroupLayouts::normal_map(),
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
    pub fn bind_group(&self, key: &str) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        self.bind_groups
            .get(&super::CacheKey { id: key.into() })
            .cloned()
    }
    pub fn bind_group_for(
        &mut self,
        texture_manager: &TextureManager,
        key: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<std::sync::Arc<wgpu::BindGroup>> {
        let binding = crate::GPU::get();
        let cache_key: crate::CacheKey = key.into();
        if let Ok(gpu) = binding.read() {
            if !self.bind_groups.contains(&super::CacheKey {
                id: cache_key.id.clone(),
            }) {
                let tex = texture_manager.get(key)?;
                let bind_group: std::sync::Arc<wgpu::BindGroup> = gpu
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("tex_bg:{}", key)),
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
                self.bind_groups.insert(cache_key.clone(), bind_group);
            }
        }

        self.bind_groups.get(&cache_key).cloned()
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
