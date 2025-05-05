use crate::{BindGroupLayouts, Camera, World};

pub trait Renderer {
    fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &BindGroupLayouts,
        world: &mut World,
        camera: &Camera,
    );
    fn update(&mut self, dt: f32);
}
