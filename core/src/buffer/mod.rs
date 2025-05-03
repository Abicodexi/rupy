use crate::{GlyphonBufferManager, WgpuBufferManager};
pub mod glyphon_buffer;
pub mod wgpu_buffer;

pub struct BufferManager {
    pub g_buffer: GlyphonBufferManager,
    pub w_buffer: WgpuBufferManager,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            g_buffer: GlyphonBufferManager::new(),
            w_buffer: WgpuBufferManager::new(),
        }
    }
}
