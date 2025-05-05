#[warn(dead_code)]
pub struct WgpuRenderer {}

impl WgpuRenderer {
    pub fn new() -> Self {
        WgpuRenderer {}
    }
}

impl crate::Renderer for WgpuRenderer {
    fn render(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &crate::BindGroupLayouts,
        world: &mut crate::World,
        camera: &crate::camera::Camera,
    ) {
        world.render(device, queue, managers, rpass, bind_group_layouts, camera);
    }
}
