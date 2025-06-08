use crate::{DebugMode, MaterialManager, ModelManager, World};

pub trait RenderPass {
    fn render(
        &self,
        models: &mut ModelManager,
        materials: &MaterialManager,
        rpass: &mut wgpu::RenderPass,
        world: &World,
        uniform_bind_group: &wgpu::BindGroup,
        debug_mode: &DebugMode,
    );
}
