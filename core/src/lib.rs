pub mod error;
pub use error::EngineError;

pub mod gpu;
pub use gpu::context::GpuContext;

pub mod renderer;
pub use renderer::traits::Renderer;
pub use renderer::wgpu_renderer::WgpuRenderer;

pub mod surface;
pub use surface::SurfaceExt;
pub use surface::SurfaceSize;

pub mod buffer;
pub use buffer::glyphon_buffer::GlyphonBuffer;
pub use buffer::glyphon_buffer::GlyphonBufferManager;
pub use buffer::wgpu_buffer::WgpuBuffer;
pub use buffer::wgpu_buffer::WgpuBufferManager;

pub mod cache;
pub use cache::key::CacheKey;
pub use cache::storage::CacheStorage;
pub use cache::storage::HashCache;

pub mod texture;

pub mod bind_group;
pub use bind_group::BindGroupLayoutBuilder;
pub use bind_group::BindGroupLayouts;

pub mod camera;

pub mod assets;
pub use assets::asset_dir;
pub use assets::loader::AssetLoader;
pub use assets::watcher::AssetWatcher;

pub mod shader;
pub use shader::ShaderManager;

pub mod pipeline;

pub mod event_bus;
pub use event_bus::ApplicationEvent;

pub mod logger;

#[cfg(feature = "logging")]
pub use logger as rupyLogger;

#[cfg(feature = "logging")]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}
#[cfg(feature = "logging")]
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}
#[cfg(feature = "logging")]
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}
#[cfg(feature = "logging")]
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {};
}
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {};
}
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {};
}
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {};
}
