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
