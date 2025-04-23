use crate::{CacheKey, CacheKeyProvider, CacheStorage, EngineError};
use wgpu::{Buffer, BufferUsages, Device};

/// Wrapper around WGPU buffers
pub struct WgpuBuffer {
    pub buffer: Buffer,
    pub size: usize,
}

impl WgpuBuffer {
    /// Create a new GPU buffer with given data and usage flags
    pub fn from_data<T: bytemuck::Pod>(
        device: &Device,
        data: &[T],
        usage: BufferUsages,
    ) -> Result<Self, EngineError> {
        use wgpu::util::DeviceExt;
        let size = (std::mem::size_of::<T>() * data.len()) as u64;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data),
            usage,
        });
        Ok(WgpuBuffer {
            buffer,
            size: size as usize,
        })
    }

    /// Update the buffer with new data via queue write
    pub fn write_data<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, bytes);
    }
}

pub type WgpuBufferCacheType = std::collections::HashMap<CacheKey, WgpuBuffer>;
pub struct WgpuBufferCache {
    inner: WgpuBufferCacheType,
}

impl WgpuBufferCache {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn get_or_create_buffer<K, F>(&mut self, key_source: &K, create_fn: F) -> &mut WgpuBuffer
    where
        K: CacheKeyProvider,
        F: FnOnce() -> WgpuBuffer,
    {
        let key = key_source.cache_key();
        self.inner.get_or_create(key, create_fn)
    }
}
