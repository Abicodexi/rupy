use crate::{EngineError, Texture};

pub fn asset_dir() -> Result<std::path::PathBuf, crate::EngineError> {
    let curr_dir = std::env::current_dir()?;
    let assets_dir = curr_dir.join(AssetLoader::DIR);
    Ok(assets_dir)
}

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));

pub struct AssetLoader;
impl AssetLoader {
    pub const DIR: &str = "assets";
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        AssetLoader::base_path().join(rel_path)
    }

    pub fn read_text(rel_path: &str) -> Result<String, EngineError> {
        let path = AssetLoader::resolve(rel_path);
        std::fs::read_to_string(&path)
            .map_err(|e| EngineError::FileSystemError(format!("{:?}: {}", path, e)))
    }

    pub fn read_bytes(rel_path: &str) -> Result<Vec<u8>, EngineError> {
        let path = AssetLoader::resolve(rel_path);
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
    pub fn read_tobj(
        rel_path: &str,
    ) -> Result<
        (
            Vec<tobj::Model>,
            Result<Vec<tobj::Material>, tobj::LoadError>,
        ),
        EngineError,
    > {
        let base_dir = AssetLoader::base_path();
        let path = base_dir.join("models").join(rel_path);
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
    pub async fn load_texture_file(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        file: &str,
    ) -> Result<Texture, EngineError> {
        let path = &format!("textures/{}", file);
        let file_bytes = AssetLoader::read_bytes(path)?;
        let texture = Texture::from_bytes(device, queue, &file_bytes, path).await?;
        Ok(texture)
    }
}
