use std::sync::Arc;

use wgpu::{DepthStencilState, VertexBufferLayout};

use crate::{AssetService, CacheKey, EngineError};

pub fn create_render_pipeline<'a>(
    service: &AssetService,
    f_shader: &str,
    v_shader: &str,
    layout: wgpu::PipelineLayout,
    buffers: &'a [VertexBufferLayout<'a>],
    format: wgpu::TextureFormat,
    depth_stencil: Option<DepthStencilState>,
    label: String,
) -> Result<wgpu::RenderPipeline, EngineError> {
    service.load_shader(v_shader);
    service.load_shader(f_shader);
    let f_shader = service.get_shader(&CacheKey::from(f_shader)).unwrap();
    let v_shader = service.get_shader(&CacheKey::from(v_shader)).unwrap();
    let pipeline = service
        .device()
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &v_shader,
                entry_point: Some("vs_main"),
                buffers: &buffers,
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
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        });
    Ok(pipeline)
}
pub fn create_compute_pipeline<'a>(
    service: &AssetService,
    c_shader: &str,
    layout: wgpu::PipelineLayout,
    entry_point: Option<&'a str>,
    label: &str,
) -> Result<wgpu::ComputePipeline, EngineError> {
    service.load_shader(c_shader);
    let shader = service.get_shader(&CacheKey::from(c_shader)).unwrap();
    let pipeline = service
        .device()
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            cache: Default::default(),
            module: &shader,
            entry_point,
            compilation_options: Default::default(),
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
    pub fn create_pipeline<'a>(
        &mut self,
        service: &AssetService,
        c_shader: &str,
        layout: wgpu::PipelineLayout,
        entry_point: Option<&'a str>,
        key: CacheKey,
        label: &str,
    ) -> Result<(), EngineError> {
        if !self.pipelines.contains_key(&key) {
            let pipeline = create_compute_pipeline(service, c_shader, layout, entry_point, label)?;
            self.pipelines.insert(key, pipeline.into());
        }
        Ok(())
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::ComputePipeline>> for ComputePipelineManager {
    fn get_resource(
        &self,
        key: &crate::CacheKey,
    ) -> Option<&std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.get(key)
    }

    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
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
    ) -> Result<Arc<wgpu::ComputePipeline>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::ComputePipeline>, EngineError>,
    {
        self.pipelines.get_or_create(key, create_fn)
    }
    fn insert_resource(
        &mut self,
        key: crate::CacheKey,
        resource: std::sync::Arc<wgpu::ComputePipeline>,
    ) {
        self.pipelines.insert(key, resource);
    }
    fn remove_resource(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<std::sync::Arc<wgpu::ComputePipeline>> {
        self.pipelines.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<wgpu::ComputePipeline>>
    where
        std::sync::Arc<wgpu::ComputePipeline>: 'a,
    {
        self.pipelines.values()
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
    pub fn create_pipeline<'a>(
        &mut self,
        service: &AssetService,
        f_shader: &str,
        v_shader: &str,
        layout: wgpu::PipelineLayout,
        buffers: &'a [VertexBufferLayout<'a>],
        format: wgpu::TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        key: CacheKey,
        label: String,
    ) -> Result<(), EngineError> {
        if !self.pipelines.contains_key(&key) {
            let pipeline = create_render_pipeline(
                service,
                f_shader,
                v_shader,
                layout,
                buffers,
                format,
                depth_stencil,
                label,
            )?;
            self.pipelines.insert(key, pipeline.into());
        }
        Ok(())
    }
}
impl crate::CacheStorage<std::sync::Arc<wgpu::RenderPipeline>> for RenderPipelineManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.get(key)
    }

    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
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
    ) -> Result<Arc<wgpu::RenderPipeline>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::RenderPipeline>, EngineError>,
    {
        self.pipelines.get_or_create(key, create_fn)
    }
    fn insert_resource(
        &mut self,
        key: crate::CacheKey,
        resource: std::sync::Arc<wgpu::RenderPipeline>,
    ) {
        self.pipelines.insert(key, resource);
    }
    fn remove_resource(
        &mut self,
        key: &crate::CacheKey,
    ) -> Option<std::sync::Arc<wgpu::RenderPipeline>> {
        self.pipelines.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a std::sync::Arc<wgpu::RenderPipeline>>
    where
        std::sync::Arc<wgpu::RenderPipeline>: 'a,
    {
        self.pipelines.values()
    }
}
