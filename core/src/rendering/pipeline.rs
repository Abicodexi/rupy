
pub struct HDR {
    pipeline: wgpu::RenderPipeline,
}
impl HDR {
   

    fn create_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let layout =&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hdr pipeline layout"),
            bind_group_layouts: &[&crate::BindGroupLayouts::texture()],
            push_constant_ranges: &[],
        });
           device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("HDR pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: format.add_srgb_suffix(),
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
            })
       
    }
    pub fn create(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<HDR, crate::EngineError> {
        let shader = "hdr.wgsl";
        let hdr_shader = crate::Shader::load(shader)?;
        let pipeline = HDR::create_pipeline(
            device,
            &hdr_shader,
            surface_config.format,
        );
        Ok(Self {
            pipeline,
        })
    }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
   
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

    pub fn hdr(
        device:&wgpu::Device,
        cfg: &wgpu::SurfaceConfiguration,
    ) -> Result<HDR, crate::EngineError> {
        HDR::create(device, cfg)
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
    fn remove(&mut self, key: &crate::CacheKey) {
        self.pipelines.remove(key);
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
    fn remove(&mut self, key: &crate::CacheKey) {
        self.pipelines.remove(key);
    }
}
