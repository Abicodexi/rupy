use glyphon::Attrs;

use super::{GlyphonBuffer, TextRegion};

pub struct RenderText {
    buffer: GlyphonBuffer,
    font_system: glyphon::FontSystem,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    font_size: f32,
}

impl RenderText {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
        depth_stencil: &Option<wgpu::DepthStencilState>,
    ) -> Self {
        let font_size = 5.0;
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, swapchain_format);
        let multisample = wgpu::MultisampleState::default();

        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            multisample,
            depth_stencil.as_ref().cloned(),
        );

        let mut font_system = glyphon::FontSystem::new();

        let buffer = GlyphonBuffer::new(
            &mut font_system,
            Some(glyphon::Metrics::new(font_size, font_size)),
            Some(glyphon::Shaping::Basic),
            glyphon::cosmic_text::LineEnding::CrLf,
            glyphon::AttrsList::new(glyphon::Attrs::new()),
            Some(glyphon::cosmic_text::Align::Left),
            None,
        );

        RenderText {
            buffer,
            font_system,
            atlas,
            renderer,
            swash_cache,
            viewport,
            font_size,
        }
    }
    pub fn create_buffer(
        &mut self,
        metrics: Option<glyphon::Metrics>,
        shaping: Option<glyphon::Shaping>,
        ending: glyphon::cosmic_text::LineEnding,
        attrs_list: glyphon::AttrsList,
        align: Option<glyphon::cosmic_text::Align>,
        shape_opt: Option<glyphon::ShapeLine>,
    ) -> GlyphonBuffer {
        GlyphonBuffer::new(
            &mut self.font_system,
            metrics,
            shaping,
            ending,
            attrs_list,
            align,
            shape_opt,
        )
    }
    pub fn create_buffer_default(&mut self) -> GlyphonBuffer {
        GlyphonBuffer::new(
            &mut self.font_system,
            Some(glyphon::Metrics::new(self.font_size, self.font_size)),
            Some(glyphon::Shaping::Basic),
            glyphon::cosmic_text::LineEnding::CrLf,
            glyphon::AttrsList::new(glyphon::Attrs::new()),
            Some(glyphon::cosmic_text::Align::Left),
            None,
        )
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
    pub fn set_buffer_lines(&mut self, lines: Vec<glyphon::BufferLine>) {
        self.buffer.set_lines(lines);
    }
    pub fn shape_buffer(&mut self) {
        self.buffer.shape(&mut self.font_system);
    }
    pub fn prepare_regions(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        regions: &[TextRegion],
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        let buffer = &mut self.buffer;
        let mut areas: Vec<glyphon::TextArea<'_>> = Vec::new();
        let mut lines: Vec<glyphon::BufferLine> = Vec::new();
        let shaping = glyphon::Shaping::Basic;
        let ending = glyphon::cosmic_text::LineEnding::CrLf;
        let attrs_list = glyphon::AttrsList::new(Attrs::new());

        for region in regions {
            lines.push(glyphon::BufferLine::new(
                region.text.clone(),
                ending,
                attrs_list.clone(),
                shaping,
            ));
        }

        buffer.set_lines(lines);
        buffer.shape(&mut self.font_system);

        for region in regions.iter() {
            areas.push(glyphon::TextArea {
                buffer: buffer.get(),
                left: region.pos[0],
                top: region.pos[1],
                scale: self.font_size,
                bounds: region.bounds.unwrap_or(glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: surface_config.width as i32,
                    bottom: surface_config.height as i32,
                }),
                default_color: region.color,
                custom_glyphs: &[],
            });
        }

        if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            areas,
            &mut self.swash_cache,
        ) {
            crate::log_error!("Error preparing text: {}", e);
        }

        // buffers lives until the end of this function, so all TextArea refs are valid!
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
                buffer: &data.get(),
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
}

impl crate::RenderPass for RenderText {
    fn render(
        &self,
        _managers: &mut crate::ModelManager,
        rpass: &mut wgpu::RenderPass,
        _world: &crate::World,
        _uniform_bind_group: &wgpu::BindGroup,
    ) {
        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            crate::log_error!("Error rendering text: {}", e);
        }
    }
}
