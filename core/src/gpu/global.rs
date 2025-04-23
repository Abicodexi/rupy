use crate::{gpu::context::GpuContext, EngineError};
use once_cell::sync::OnceCell;

/// Global GPU context singleton
static GLOBAL_GPU: OnceCell<GpuContext> = OnceCell::new();

/// Initialize the global GPU context. Must be called once at startup.
/// Returns an error if initialization fails or if already initialized.
async fn init_global_gpu() -> Result<(), EngineError> {
    let ctx = GpuContext::new().await?;
    GLOBAL_GPU
        .set(ctx)
        .map_err(|_| EngineError::AdapterNotFound)
}

/// Retrieve a reference to the global GPU context, lazily initializing
/// via a blocking call if needed. Panics on error.
pub fn get_global_gpu() -> &'static GpuContext {
    if GLOBAL_GPU.get().is_none() {
        if let Err(e) = pollster::block_on(init_global_gpu()) {
            panic!("Failed to initialize global GPU context: {}", e);
        }
    }
    GLOBAL_GPU
        .get()
        .expect("Global GPU context not initialized after block_on")
}
