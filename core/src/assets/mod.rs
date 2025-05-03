use std::path::PathBuf;

use loader::AssetLoader;
use wgpu::ShaderModule;
pub mod loader;

pub const DIR_ASSETS: &str = "assets";

pub fn asset_dir() -> PathBuf {
    let dir_path = std::env::current_dir().unwrap().join(DIR_ASSETS);
    dir_path
}

pub struct Assets<'a> {
    loader: &'a AssetLoader,
}

impl<'a> Assets<'a> {
    pub fn new(loader: &'a AssetLoader) -> Self {
        Self { loader }
    }

    pub fn shader(&self, path: &str) -> ShaderModule {
        self.loader
            .load_shader(path)
            .expect("Failed to load shader")
    }

    pub fn text(&self, path: &str) -> String {
        self.loader
            .load_text(path)
            .expect("Failed to load text file")
    }
}
