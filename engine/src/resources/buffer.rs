pub struct BufferManager {
    pub g_buffer: crate::GlyphonBufferManager,
    pub w_buffer: crate::WgpuBufferManager,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            g_buffer: crate::GlyphonBufferManager::new(),
            w_buffer: crate::WgpuBufferManager::new(),
        }
    }
}
