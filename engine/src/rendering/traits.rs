pub trait Renderer {
    fn render(
        &self,
        models: &mut crate::ModelManager,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        uniform_bind_group: &wgpu::BindGroup,
    );
}
