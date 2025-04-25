use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("GPU error: {0}")]
    GpuError(String),

    #[error("RwLock error: {0}")]
    RwLockError(String),

    #[error("No suitable GPU adapter found")]
    AdapterNotFound,

    #[error("Failed to request device: {0}")]
    DeviceRequestFailed(#[from] wgpu::RequestDeviceError),

    #[error("OS error creating window: {0}")]
    Os(#[from] winit::error::OsError),

    #[error("Failed to create render surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("Surface config error: {0}")]
    SurfaceConfigError(String),

    #[error("WGPU buffer error: {0}")]
    WgpuBufferError(String),

    #[error("Glyphon buffer error: {0}")]
    GlyphonBufferError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("FileSystem error: {0}")]
    FileSystemError(String),
}
