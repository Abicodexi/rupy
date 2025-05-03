use super::asset_dir;
use crate::{texture::Texture, EngineError};
use std::{path::PathBuf, sync::Arc};

pub struct AssetLoader {
    base_path: PathBuf,
    device: Arc<wgpu::Device>,
}

impl AssetLoader {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        let base_path = asset_dir();
        Self { device, base_path }
    }

    pub fn resolve(&self, rel_path: &str) -> PathBuf {
        self.base_path.join(rel_path)
    }

    pub fn load_text(&self, rel_path: &str) -> Result<String, EngineError> {
        let path = self.resolve(rel_path);
        std::fs::read_to_string(&path)
            .map_err(|e| EngineError::FileSystemError(format!("Failed to read {:?}: {}", path, e)))
    }

    pub fn load_shader(&self, rel_path: &str) -> Result<wgpu::ShaderModule, EngineError> {
        let path = self.resolve(&format!("shaders\\{}", rel_path));

        let shader_source = std::fs::read_to_string(&path)?;

        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(rel_path),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        Ok(shader_module)
    }
    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub async fn load_texture(
        &self,
        queue: &wgpu::Queue,
        rel_path: &str,
    ) -> Result<Texture, EngineError> {
        let path = self.resolve(&format!("textures\\{}", rel_path));

        let bytes = Self::read_bytes(&path)?;
        let tex = Texture::from_bytes(&self.device, queue, &bytes, path).await?;
        Ok(tex)
    }
}
