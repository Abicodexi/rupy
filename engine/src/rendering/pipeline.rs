use crate::{EngineError, RenderBindGroupLayouts, Vertex2d};

pub fn create_hdr_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, EngineError> {
    let v_shader = crate::Shader::load("hdr.vert.wgsl")?;
    let f_shader = crate::Shader::load("hdr.frag.wgsl")?;

    let pipeline_layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hdr pipeline layout"),
        bind_group_layouts: &[&RenderBindGroupLayouts::texture()],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("HDR pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &v_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &f_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,

        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });
    Ok(pipeline)
}
pub fn create_sprite2d_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Result<wgpu::RenderPipeline, EngineError> {
    let f_shader = crate::Shader::load("sprite2d.frag.wgsl")?;
    let v_shader = crate::Shader::load("sprite2d.vert.wgsl")?;
    let ortho_bind_group_layout = crate::RenderBindGroupLayouts::ortho_uniform();
    let texture_bind_group_layout = crate::RenderBindGroupLayouts::texture();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("spride 2d pipeline layout")),
        bind_group_layouts: &[ortho_bind_group_layout, texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("sprite 2d pipeline",)),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &v_shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex2d::LAYOUT],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &f_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::OVER,
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: Default::default(),
    });
    Ok(pipeline)
}

pub struct PipelineManager {
    pub render: crate::RenderPipelineManager,
    pub compute: crate::ComputePipelineManager,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            render: crate::RenderPipelineManager::new(),
            compute: crate::ComputePipelineManager::new(),
        }
    }
}
pub struct ComputePipelineManager {
    pipelines: crate::HashCache<std::sync::Arc<wgpu::ComputePipeline>>,
}
impl ComputePipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: crate::HashCache::new(),
        }
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::ComputePipeline>> for ComputePipelineManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<&mut std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::ComputePipeline>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::ComputePipeline>,
    {
        let start = std::time::Instant::now();
        let pipeline = self.pipelines.entry(key).or_insert_with(create_fn);
        crate::log_debug!("Loaded in {:.2?}", start.elapsed());
        pipeline
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::ComputePipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.remove(key)
    }
}
pub struct RenderPipelineManager {
    pipelines: crate::HashCache<std::sync::Arc<wgpu::RenderPipeline>>,
}
impl RenderPipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: crate::HashCache::new(),
        }
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::RenderPipeline>> for RenderPipelineManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.get(key)
    }

    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<&mut std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> &mut std::sync::Arc<wgpu::RenderPipeline>
    where
        F: FnOnce() -> std::sync::Arc<wgpu::RenderPipeline>,
    {
        let start = std::time::Instant::now();
        let pipeline = self.pipelines.entry(key).or_insert_with(create_fn);
        crate::log_debug!("Loaded in {:.2?}", start.elapsed());
        pipeline
    }
    fn insert(&mut self, key: crate::CacheKey, resource: std::sync::Arc<wgpu::RenderPipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) -> Option<std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.remove(key)
    }
}
