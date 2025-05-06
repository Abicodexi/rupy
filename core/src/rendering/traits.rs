pub trait Renderer {
    fn render(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
    );
}
