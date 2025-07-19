use std::path::PathBuf;

use image::DynamicImage;

use crate::{EngineError, Texture};

pub fn asset_dir() -> Result<std::path::PathBuf, crate::EngineError> {
    let curr_dir = std::env::current_dir()?;

    let assets_dir = curr_dir.join(AssetLoader::DIR);
    crate::log_debug!("Current dir: {:?}", assets_dir);
    Ok(assets_dir)
}

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| asset_dir().expect("couldn’t find asset dir"));

pub struct AssetLoader;
impl AssetLoader {
    pub const DIR: &str = "assets";
    pub fn base_path() -> &'static std::path::PathBuf {
        &BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        AssetLoader::base_path().join(rel_path)
    }

    pub fn text(path: PathBuf) -> Result<String, EngineError> {
        std::fs::read_to_string(&path)
            .map_err(|e| EngineError::FileSystemError(format!("{:?}: {}", path, e)))
    }

    pub fn bytes(path: PathBuf) -> Result<Vec<u8>, EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub fn tobj(
        path: PathBuf,
    ) -> Result<
        (
            Vec<tobj::Model>,
            Result<Vec<tobj::Material>, tobj::LoadError>,
        ),
        EngineError,
    > {
        match tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        ) {
            Ok(result) => Ok(result),
            Err(e) => Err(EngineError::TobjLoadError(e)),
        }
    }
    pub fn image(path: PathBuf) -> Result<DynamicImage, EngineError> {
        image::open(path).map_err(|e| EngineError::AssetLoadError(e.to_string()))
    }
    pub async fn texture(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        file: &str,
    ) -> Result<Texture, EngineError> {
        let path = AssetLoader::resolve("textures").join(file);
        let file_bytes = AssetLoader::bytes(path)?;
        let texture = Texture::from_bytes(device, queue, &file_bytes, file).await?;
        Ok(texture)
    }
    pub async fn shader(
        device: &wgpu::Device,
        file: &str,
    ) -> Result<wgpu::ShaderModule, EngineError> {
        let path = AssetLoader::resolve("shaders").join(file);

        let shader_source = std::fs::read_to_string(&path)?;
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(file),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        Ok(shader_module)
    }
}
