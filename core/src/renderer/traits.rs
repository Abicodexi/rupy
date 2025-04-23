use crate::{
    camera::uniform::CameraUniform, texture::TextureManager, BindGroupLayouts, GpuContext, Mesh,
    WgpuBufferCache,
};
use wgpu::SurfaceTexture;

pub trait Renderer {
    fn resize(&mut self, new_config: &wgpu::SurfaceConfiguration);

    fn render(
        &self,
        gpu: &GpuContext,
        surface_texture: SurfaceTexture,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &TextureManager,
        wgpu_buffer_cache: &mut WgpuBufferCache,
        camera_uniform: &CameraUniform,
    );
    fn render_mesh(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera_bg: &wgpu::BindGroup,
        texture_bg: &wgpu::BindGroup,
        wgpu_buffer_cache: &mut WgpuBufferCache,
        mesh: &Mesh,
    );
    fn update(&mut self, dt: f32);
}
