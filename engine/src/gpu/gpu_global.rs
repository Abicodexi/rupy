use crate::gpu::gpu_builder::GpuBuilder;
use crate::GPU;

static GPU_GLOBAL: std::sync::OnceLock<std::sync::Arc<std::sync::RwLock<GPU>>> =
    std::sync::OnceLock::new();

pub fn get_global_gpu() -> std::sync::Arc<std::sync::RwLock<GPU>> {
    GPU_GLOBAL
        .get()
        .expect("Global gpu is not initialized")
        .clone()
}
pub fn init_global_gpu(builder: GpuBuilder) {
    let gpu = builder.build();
    let arc = std::sync::Arc::new(std::sync::RwLock::new(gpu));
    GPU_GLOBAL.set(arc).expect("GPU already initialized");
}
