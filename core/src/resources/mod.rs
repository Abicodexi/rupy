pub mod bind_group;
pub use bind_group::*;

pub mod buffer;
pub use buffer::*;

pub mod cache;
pub use cache::*;

pub mod cache_key;
pub use cache_key::*;

pub mod texture;
pub use texture::*;

pub struct Resources {
    pub gpu: std::sync::Arc<crate::GpuContext>,
    pub asset_loader: std::sync::Arc<crate::AssetLoader>,
}

pub struct Managers {
    pub shader_manager: crate::ShaderManager,
    pub pipeline_manager: crate::PipelineManager,
    pub buffer_manager: BufferManager,
    pub texture_manager: TextureManager,
    pub mesh_manager: crate::MeshManager,
    pub material_manager: crate::MaterialManager,
    pub model_manager: crate::ModelManager,
}

impl Managers {
    pub fn render_models(&self, rpass: &mut wgpu::RenderPass) {
        self.model_manager.render(
            rpass,
            &self.pipeline_manager,
            &self.buffer_manager,
            &self.material_manager,
            &self.mesh_manager,
        );
    }
}
