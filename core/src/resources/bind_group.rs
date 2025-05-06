use super::{CacheStorage, HashCache, TextureManager};

static BGL: once_cell::sync::OnceCell<BindGroupLayouts> = once_cell::sync::OnceCell::new();

fn init_bind_group_layouts(device: &wgpu::Device) {
    BGL.get_or_init(|| BindGroupLayouts::new(device));
}

fn texture_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").texture
}

fn camera_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").camera
}
fn equirect_src_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").equirect_src
}
fn equirect_dst_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    &BGL.get().expect("BGL not initialized").equirect_dst
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
    pub texture: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub equirect_src: wgpu::BindGroupLayout,
    pub equirect_dst: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    pub fn init(device: &wgpu::Device) {
        init_bind_group_layouts(device);
    }
    pub fn camera() -> &'static wgpu::BindGroupLayout {
        camera_bind_group_layout()
    }
    pub fn texture() -> &'static wgpu::BindGroupLayout {
        texture_bind_group_layout()
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

        let camera = BindGroupLayoutBuilder::new(&device)
            .label("camera bind group layout")
            .binding(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
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

        Self {
            texture,
            camera,
            equirect_src,
            equirect_dst,
        }
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
