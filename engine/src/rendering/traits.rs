pub trait Renderer {
    fn render(
        &self,
        models: &mut crate::ModelManager,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
        light: &crate::Light,
        uniform_bind_group: &wgpu::BindGroup,
    );
}
