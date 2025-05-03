use glyphon::{
    Cache, FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
    Viewport,
};
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};

use crate::{log_error, GlyphonBuffer};

pub struct GlyphonRender {
    font_system: FontSystem,
    atlas: TextAtlas,
    renderer_2d: TextRenderer,
    renderer_3d: TextRenderer,
    swash_cache: SwashCache,
    viewport: Viewport,
}

impl GlyphonRender {
    pub fn new(
        device: &Device,
        queue: &Queue,
        swapchain_format: wgpu::TextureFormat,
        depth_stencil: &wgpu::DepthStencilState,
    ) -> Self {
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, swapchain_format);

        let renderer_2d =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let renderer_3d = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            Some(depth_stencil.clone()),
        );

        let font_system = FontSystem::new();

        GlyphonRender {
            font_system,
            atlas,
            renderer_2d,
            renderer_3d,
            swash_cache,
            viewport,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, resolution: glyphon::Resolution) {
        self.viewport.update(queue, resolution);
    }
    pub fn prepate(
        &mut self,
        device: &Device,
        queue: &Queue,
        data: GlyphonBuffer,
        surface_config: &SurfaceConfiguration,
        use_depth: bool,
    ) {
        let _ = if use_depth {
            let _ = if let Err(e) = self.renderer_3d.prepare(
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
                log_error!("Error preparing 3d text: {}", e);
            };
        } else {
            if let Err(e) = self.renderer_2d.prepare(
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
                log_error!("Error preparing 2d text: {}", e);
            }
        };
    }
    pub fn render<'a>(&'a mut self, pass: &mut RenderPass<'a>, use_depth: bool) {
        let _ = if use_depth {
            if let Err(e) = self.renderer_3d.render(&self.atlas, &self.viewport, pass) {
                log_error!("Error rendering 3d text: {}", e);
            }
        } else {
            if let Err(e) = self.renderer_2d.render(&self.atlas, &self.viewport, pass) {
                log_error!("Error rendering 2d text: {}", e);
            }
        };
    }
}
