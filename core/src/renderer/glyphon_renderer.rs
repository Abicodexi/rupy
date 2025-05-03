use glyphon::{
    Cache, FontSystem, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::{log_error, GlyphonBuffer};

pub struct GlyphonRenderer {
    pub font_system: FontSystem,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
}

impl GlyphonRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
        depth_stencil: &wgpu::DepthStencilState,
    ) -> Self {
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, swapchain_format);

        let renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            Some(depth_stencil.clone()),
        );

        let font_system = FontSystem::new();

        GlyphonRenderer {
            font_system,
            atlas,
            renderer,
            swash_cache,
            viewport,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, resolution: glyphon::Resolution) {
        self.viewport.update(queue, resolution);
    }
    pub fn prepate(
        &mut self,
        device: &Device,
        queue: &Queue,
        data: &mut GlyphonBuffer,
        surface_config: &SurfaceConfiguration,
    ) {
        let _ = if let Err(e) = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: &data.buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: surface_config.width as i32,
                    bottom: surface_config.height as i32,
                },
                default_color: glyphon::Color::rgb(1, 1, 1),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ) {
            log_error!("Error preparing text: {}", e);
        };
    }
    pub fn render<'a>(&'a mut self, rpass: &mut wgpu::RenderPass) {
        if let Err(e) = self.renderer.render(&self.atlas, &self.viewport, rpass) {
            log_error!("Error rendering text: {}", e);
        }
    }
}
