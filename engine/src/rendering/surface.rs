/// Extends wgpu::Surface with setup and usage methods.
pub trait SurfaceExt {
    /// Resizes the surface.
    fn resize(
        &self,
        device: &wgpu::Device,
        config: &mut wgpu::SurfaceConfiguration,
        new_size: winit::dpi::PhysicalSize<u32>,
    );

    /// Configures the surface using the provided device and configuration.
    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);

    /// Returns the current surface size.
    fn size(config: &wgpu::SurfaceConfiguration) -> SurfaceSize;

    /// Acquires the next texture for rendering.
    fn texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;
}

impl<'a> SurfaceExt for wgpu::Surface<'a> {
    fn resize(
        &self,
        device: &wgpu::Device,
        config: &mut wgpu::SurfaceConfiguration,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        config.width = new_size.width.max(1);
        config.height = new_size.height.max(1);
        self.configure(device, config);
    }

    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.configure(device, config);
    }

    fn size(config: &wgpu::SurfaceConfiguration) -> SurfaceSize {
        SurfaceSize(config.width, config.height)
    }

    fn texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.get_current_texture()
    }
}

/// Encapsulates width/height helper conversions for surfaces.
pub struct SurfaceSize(pub u32, pub u32);
impl SurfaceSize {
    pub fn as_physical_size_f32(&self) -> winit::dpi::PhysicalSize<f32> {
        winit::dpi::PhysicalSize::new(self.0 as f32, self.1 as f32)
    }
    pub fn as_physical_size_u32(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.0, self.1)
    }
    pub fn width_u32(&self) -> u32 {
        self.0
    }
    pub fn height_u32(&self) -> u32 {
        self.1
    }
    pub fn to_normalized(&self, x: f32, y: f32) -> (f32, f32) {
        (x / self.0 as f32, y / self.1 as f32)
    }
    pub fn to_physical(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.0 as f32, y * self.1 as f32)
    }
}
