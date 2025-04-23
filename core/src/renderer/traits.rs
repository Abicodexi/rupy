use crate::{texture::TextureManager, BindGroupLayouts, GpuContext};
use wgpu::SurfaceTexture;

pub trait Renderer {
    fn resize(&mut self, new_config: &wgpu::SurfaceConfiguration);

    fn render(
        &self,
        gpu: &GpuContext,
        surface_texture: SurfaceTexture,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &TextureManager,
        camera_buffer: &wgpu::Buffer,
    );

    fn update(&mut self, dt: f32);
}
