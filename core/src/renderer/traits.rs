use super::Mesh;
use crate::{texture::TextureManager, BindGroupLayouts, GpuContext, WgpuBufferManager};
use wgpu::SurfaceTexture;

pub trait Renderer {
    fn resize(&mut self, new_config: &wgpu::SurfaceConfiguration, device: &wgpu::Device);

    fn render(
        &self,
        gpu: &GpuContext,
        surface_texture: SurfaceTexture,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &mut TextureManager,
        w_buffer_manager: &mut WgpuBufferManager,
        camera_bind_group: &wgpu::BindGroup,
        mesh: &Mesh,
    );
    fn update(&mut self, dt: f32);
}
