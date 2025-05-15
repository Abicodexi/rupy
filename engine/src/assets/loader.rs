use crate::EngineError;

static BASE_PATH: once_cell::sync::Lazy<std::path::PathBuf> =
    once_cell::sync::Lazy::new(|| super::asset_dir().expect("couldn’t find asset dir"));

pub struct Asset;
impl Asset {
    pub fn base_path() -> &'static std::path::PathBuf {
        &*BASE_PATH
    }
    pub fn resolve(rel_path: &str) -> std::path::PathBuf {
        Asset::base_path().join(rel_path)
    }

    pub fn read_text(rel_path: &str) -> Result<String, EngineError> {
        let path = Asset::resolve(rel_path);
        std::fs::read_to_string(&path)
            .map_err(|e| EngineError::FileSystemError(format!("{:?}: {}", path, e)))
    }

    pub fn read_bytes<P: AsRef<std::path::Path>>(path: &P) -> Result<Vec<u8>, EngineError> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
}
