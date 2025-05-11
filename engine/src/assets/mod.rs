pub mod loader;
pub use loader::*;

pub mod watcher;
pub use watcher::*;

pub const DIR_ASSETS: &str = "assets";

pub fn asset_dir() -> Result<std::path::PathBuf, crate::EngineError> {
    let curr_dir = std::env::current_dir()?;
    let assets_dir = curr_dir.join(DIR_ASSETS);
    Ok(assets_dir)
}
