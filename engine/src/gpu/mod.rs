pub mod gpu_builder;
pub use gpu_builder::GpuBuilder;

pub mod gpu_global;
pub use gpu_global::init_global_gpu;

#[derive(Debug)]
pub struct GPU {
    pub instance: std::sync::Arc<wgpu::Instance>,
    pub adapter: std::sync::Arc<wgpu::Adapter>,
    pub device: std::sync::Arc<wgpu::Device>,
    pub queue: std::sync::Arc<wgpu::Queue>,
}

pub fn select_best_backend() -> wgpu::Backends {
    #[cfg(target_os = "windows")]
    let preferred = wgpu::Backends::DX12 | wgpu::Backends::VULKAN;

    #[cfg(target_os = "macos")]
    let preferred = wgpu::Backends::METAL;

    #[cfg(target_os = "linux")]
    let preferred = wgpu::Backends::VULKAN | wgpu::Backends::GL;

    // Fallback for any weird platform
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let preferred = wgpu::Backends::all();

    preferred
}

pub fn select_available_backend() -> wgpu::Backends {
    // Try Vulkan first
    let vulkan_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::empty(),
        ..Default::default()
    });
    if !vulkan_instance
        .enumerate_adapters(wgpu::Backends::VULKAN)
        .is_empty()
    {
        crate::log_debug!("✅ Vulkan available");
        return wgpu::Backends::VULKAN;
    }

    // Fallback to GL
    let gl_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::empty(),
        ..Default::default()
    });

    if !gl_instance
        .enumerate_adapters(wgpu::Backends::GL)
        .is_empty()
    {
        crate::log_debug!("⚠️ Vulkan missing, falling back to GL");
        return wgpu::Backends::GL;
    }

    crate::log_debug!("No backends found, falling back to all()");
    wgpu::Backends::all()
}
