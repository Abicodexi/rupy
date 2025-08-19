use std::sync::Arc;
use wgpu::{DepthStencilState, VertexBufferLayout};

use crate::{AssetService, CacheKey, EngineError};

/// Create a graphics (render) pipeline from VS+FS shaders.
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

    let pipeline = service.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &v_shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &f_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

/// Cache manager for render pipelines.
pub struct RenderPipelineManager {
    pipelines: crate::HashCache<Arc<wgpu::RenderPipeline>>,
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

impl crate::CacheStorage<Arc<wgpu::RenderPipeline>> for RenderPipelineManager {
    fn get_resource(&self, key: &CacheKey) -> Option<&Arc<wgpu::RenderPipeline>> {
        self.pipelines.get(key)
    }
    fn contains_resource(&self, key: &CacheKey) -> bool {
        self.pipelines.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut Arc<wgpu::RenderPipeline>> {
        self.pipelines.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: CacheKey,
        create_fn: F,
    ) -> Result<Arc<wgpu::RenderPipeline>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::RenderPipeline>, EngineError>,
    {
        self.pipelines.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: CacheKey, resource: Arc<wgpu::RenderPipeline>) {
        self.pipelines.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &CacheKey) -> Option<Arc<wgpu::RenderPipeline>> {
        self.pipelines.remove(key)
    }
    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<wgpu::RenderPipeline>>
    where
        Arc<wgpu::RenderPipeline>: 'a,
    {
        self.pipelines.values()
    }
}

