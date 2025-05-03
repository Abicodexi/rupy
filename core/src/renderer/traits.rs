use super::Mesh;
use crate::{texture::TextureManager, BindGroupLayouts, WgpuBufferManager};

pub trait Renderer {
    fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &BindGroupLayouts,
        texture_manager: &mut TextureManager,
        w_buffer_manager: &mut WgpuBufferManager,
        camera_bind_group: &wgpu::BindGroup,
        mesh: &Mesh,
    );
    fn update(&mut self, dt: f32);
}
