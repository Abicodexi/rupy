pub struct HDR {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}
impl HDR {
    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }
    fn create_texture(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> crate::Texture {
        let hdr_sampler = HDR::create_sampler(device);
        let hdr_texture = crate::Texture::new(
            device,
            wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            surface_config.format.add_srgb_suffix(),
            1,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some(wgpu::AddressMode::ClampToEdge),
            wgpu::FilterMode::Nearest,
            Some(hdr_sampler),
            Some("hdr"),
        );
        hdr_texture
    }
    fn create_pipeline_layout(device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hdr pipeline layout"),
            bind_group_layouts: &[&crate::BindGroupLayouts::texture()],
            push_constant_ranges: &[],
        })
    }
    fn create_pipeline(
        device: &wgpu::Device,
        texture: &crate::Texture,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&texture.label),
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
        let hdr_texture = HDR::create_texture(device, surface_config);
        let hdr_bind_group = crate::BindGroup::hdr(device, &hdr_texture, "hdr");
        let hdr_pipeline_layout = HDR::create_pipeline_layout(device);
        let hdr_shader = crate::Asset::shader("hdr.wgsl")?;
        let pipeline = HDR::create_pipeline(
            device,
            &hdr_texture,
            &hdr_pipeline_layout,
            &hdr_shader,
            surface_config.format,
        );
        Ok(Self {
            pipeline,
            bind_group: hdr_bind_group,
        })
    }
    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hdr compute pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
    }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
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
        &mut self,
        device: &wgpu::Device,
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
