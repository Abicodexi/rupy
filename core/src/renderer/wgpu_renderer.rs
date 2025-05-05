use crate::{BindGroupLayouts, Camera, Renderer, World};

#[warn(dead_code)]
pub struct WgpuRenderer {}

impl WgpuRenderer {
    pub fn new() -> Self {
        WgpuRenderer {}
    }
}

impl Renderer for WgpuRenderer {
    fn update(&mut self, _dt: f32) {}

    fn render(
        &self,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &BindGroupLayouts,
        world: &mut World,
        camera: &Camera,
    ) {
        world.render_environment(rpass, bind_group_layouts, &camera.bind_group);
        world.render_entities(rpass, camera.frustum);
    }
}
