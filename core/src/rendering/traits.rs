pub trait Renderer {
    fn render(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        bind_group_layouts: &crate::BindGroupLayouts,
        world: &mut crate::World,
        camera: &crate::camera::Camera,
    );
}
