use super::TextRegion;
use crate::gfx::buffer::GlyphonBuffer;
use glyphon::{
    cosmic_text::{Align, LineEnding},
    Attrs, AttrsList, BufferLine, Color, FontSystem, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport,
};

pub struct GlyphonTextRenderer {
    buffer: GlyphonBuffer,

    font_system: FontSystem,
    swash_cache: SwashCache,

    atlas: TextAtlas,

    renderer: TextRenderer,

    viewport: Viewport,

    font_size: f32,

    regions: Vec<TextRegion>,
}

impl GlyphonTextRenderer {
    pub const LINE_ENDING: LineEnding = LineEnding::Lf;
    pub const ALIGNMENT: Align = Align::Left;
    pub const SHAPING: Shaping = Shaping::Basic;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
    ) -> Self {
        let font_size = 5.0;
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, swapchain_format);
        let multisample = wgpu::MultisampleState::default();

        let renderer = glyphon::TextRenderer::new(&mut atlas, device, multisample, None);

        let mut font_system = glyphon::FontSystem::new();

        let buffer = GlyphonBuffer::new(
            &mut font_system,
            Some(glyphon::Metrics::new(font_size, font_size)),
            Some(Self::SHAPING),
            Self::LINE_ENDING,
            glyphon::AttrsList::new(glyphon::Attrs::new()),
            Some(Self::ALIGNMENT),
            None,
        );
        let regions = Vec::new();

        GlyphonTextRenderer {
            buffer,
            font_system,
            swash_cache,
            atlas,
            renderer,
            viewport,
            font_size,
            regions,
        }
    }
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
    pub fn buffer(&self) -> &GlyphonBuffer {
        &self.buffer
    }
    pub fn set_metrics(&mut self, font_size: f32, line_height: f32) {
        self.font_size = font_size;
        self.buffer.get_mut().set_metrics(
            &mut self.font_system,
            glyphon::Metrics {
                font_size,
                line_height,
                ..Default::default()
            },
        );
    }
    pub fn shape(&mut self) {
        self.buffer.shape(&mut self.font_system);
    }
    pub fn set_lines(&mut self, lines: Vec<BufferLine>) {
        self.buffer.set_lines(lines);
    }
    pub fn queue_text(&mut self, text: impl Into<String>, x: f32, y: f32, color: glyphon::Color) {
        let region = TextRegion {
            text: text.into(),
            pos: [x, y],
            color,
            bounds: None,
        };
        self.regions.push(region);
    }

    pub fn queue_region(&mut self, region: TextRegion) {
        self.regions.push(region);
    }

    pub fn clear_regions(&mut self) {
        self.regions.clear();
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: f32, height: f32) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: width as u32,
                height: height as u32,
            },
        );
    }

    fn prepare_all(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        let mut areas = Vec::with_capacity(self.regions.len());
        let mut buffers = Vec::with_capacity(self.regions.len());

        for region in &self.regions {
            let mut buffer = GlyphonBuffer::new(
                &mut self.font_system,
                Some(glyphon::Metrics::new(self.font_size, self.font_size)),
                Some(Self::SHAPING),
                Self::LINE_ENDING,
                AttrsList::new(Attrs::new()),
                Some(Self::ALIGNMENT),
                None,
            );
            buffer.set_lines(vec![BufferLine::new(
                region.text.clone(),
                Self::LINE_ENDING,
                AttrsList::new(Attrs::new()),
                Self::SHAPING,
            )]);
            buffer.shape(&mut self.font_system);

            buffers.push(buffer);
        }
        for (region, buffer) in self.regions.iter().zip(&buffers) {
            areas.push(TextArea {
                buffer: buffer.get(),
                left: region.pos[0],
                top: region.pos[1],
                scale: self.font_size,
                bounds: region.bounds.unwrap_or(TextBounds {
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
            crate::log_error!("RenderText::prepare_all() error: {}", e);
        }
    }

    pub fn draw_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rpass: &mut wgpu::RenderPass<'_>,
        x: f32,
        y: f32,
        bounds: TextBounds,
    ) {
        if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: self.buffer.get(),
                left: x,
                top: y,
                scale: self.font_size,
                bounds,
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ) {
            crate::log_error!("RenderText::draw_custom_buffer() error: {}", e);
        }
        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            crate::log_error!("RenderText::render() error: {}", e);
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_config: &wgpu::SurfaceConfiguration,
        rpass: &mut wgpu::RenderPass<'_>,
    ) {
        self.prepare_all(device, queue, surface_config);

        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            crate::log_error!("RenderText::draw() error: {}", e);
        }

        self.clear_regions();
    }
}
