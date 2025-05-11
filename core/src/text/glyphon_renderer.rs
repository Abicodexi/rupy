pub struct GlyphonRenderer {
    pub font_system: glyphon::FontSystem,
    pub atlas: glyphon::TextAtlas,
    pub renderer: glyphon::TextRenderer,
    pub swash_cache: glyphon::SwashCache,
    pub viewport: glyphon::Viewport,
}

impl GlyphonRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
        depth_stencil: &Option<wgpu::DepthStencilState>,
    ) -> Self {
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, swapchain_format);

        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            depth_stencil.as_ref().cloned(),
        );

        let font_system = glyphon::FontSystem::new();

        GlyphonRenderer {
            font_system,
            atlas,
            renderer,
            swash_cache,
            viewport,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, new_size: winit::dpi::PhysicalSize<u32>) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: new_size.width,
                height: new_size.height,
            },
        );
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &mut super::GlyphonBuffer,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        let _ = if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [glyphon::TextArea {
                buffer: &data.buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: surface_config.width as i32,
                    bottom: surface_config.height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ) {
            crate::log_error!("Error preparing text: {}", e);
        };
    }
    pub fn render<'a>(&'a mut self, rpass: &mut wgpu::RenderPass) {
        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            crate::log_error!("Error rendering text: {}", e);
        }
    }
}

impl crate::Renderer for GlyphonRenderer {
    fn render(
        &self,
        _managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        _world: &crate::World,
        _camera: &crate::camera::Camera,
        _uniform_bind_group: &crate::Light,
        uniform_bind_group: &wgpu::BindGroup,
    ) {
        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            crate::log_error!("Error rendering text: {}", e);
        }
    }
}
