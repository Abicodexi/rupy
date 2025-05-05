use std::sync::Arc;

use crate::{CacheKey, CacheStorage, HashCache};
use wgpu::{Buffer, BufferUsages, Device};

/// Wrapper around WGPU buffers
pub struct WgpuBuffer {
    pub buffer: Buffer,
    pub size: usize,
}

impl WgpuBuffer {
    /// Create a new GPU buffer with given data and usage flags
    pub fn from_data<T: bytemuck::Pod>(
        queue: &wgpu::Queue,
        device: &Device,
        data: &[T],
        usage: BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let size = (std::mem::size_of::<T>() * data.len()) as u64;
        let contents = bytemuck::cast_slice(data);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents,
            usage,
        });
        queue.write_buffer(&buffer, 0, contents);
        WgpuBuffer {
            buffer,
            size: size as usize,
        }
    }
    /// Create a new empty GPU buffer with given usage flags

    pub fn new_empty(device: &Device, usage: BufferUsages) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &[],
            usage,
        });
        WgpuBuffer { buffer, size: 0 }
    }
    /// Update the buffer with new data via queue write
    pub fn write_data<T: bytemuck::Pod>(
        &mut self,
        queue: &wgpu::Queue,
        data: &[T],
        offset: Option<u64>,
    ) {
        let bytes = bytemuck::cast_slice(data);
        let size = (std::mem::size_of::<T>() * data.len()) as u64;
        self.size = size as usize;
        queue.write_buffer(&self.buffer, offset.unwrap_or(0), bytes);
    }
}

pub type WgpuBufferCacheType = HashCache<WgpuBuffer>;
pub struct WgpuBufferManager {
    inner: WgpuBufferCacheType,
}

impl WgpuBufferManager {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
    // pub fn get(&self, key_source: &CacheKey) -> Option<&WgpuBuffer> {
    //     self.inner.get(key_source)
    // }
    // pub fn get_or_create<F>(&mut self, key_source: &CacheKey, create_fn: F) -> &mut WgpuBuffer
    // where
    //     F: FnOnce() -> WgpuBuffer,
    // {
    //     self.inner.get_or_create(key_source.clone(), create_fn)
    // }
}

impl CacheStorage<WgpuBuffer> for WgpuBufferManager {
    fn get<K: Into<CacheKey>>(&self, key: K) -> Option<&WgpuBuffer> {
        self.inner.get(&key.into())
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.inner.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut WgpuBuffer> {
        self.inner.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut WgpuBuffer
    where
        F: FnOnce() -> WgpuBuffer,
    {
        self.inner.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: WgpuBuffer) {
        self.inner.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.inner.remove(key);
    }
}
