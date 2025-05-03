use crate::EngineError;
use std::path::PathBuf;
pub mod loader;
pub mod watcher;

pub const DIR_ASSETS: &str = "assets";

pub fn asset_dir() -> Result<PathBuf, EngineError> {
    let curr_dir = std::env::current_dir()?;
    let assets_dir = curr_dir.join(DIR_ASSETS);
    Ok(assets_dir)
}
