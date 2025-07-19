use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::EngineError;

/// Wrapper around WGPU buffers
#[derive(Debug)]
pub struct WgpuBuffer {
    buffer: wgpu::Buffer,
    size: usize,
    usage: wgpu::BufferUsages,
    label: String,
}

impl WgpuBuffer {
    /// Create a new GPU buffer with given data and usage flags
    pub fn from_data<T: bytemuck::Pod>(
        device: &wgpu::Device,
        data: &[T],
        usage: wgpu::BufferUsages,
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
        crate::WgpuBuffer {
            buffer,
            size: size as usize,
            usage,
            label: label.unwrap_or("unnamed").to_string(),
        }
    }
    pub fn get(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn size(&self) -> usize {
        self.size
    }

    /// Allocate a GPU buffer with a fixed byte‐capacity (uninitialized),
    /// using the given usage flags. `capacity` is in bytes.
    pub fn with_capacity(
        device: &wgpu::Device,
        capacity: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: capacity,
            usage,
            mapped_at_creation: false,
        });
        WgpuBuffer {
            buffer,
            size: capacity as usize,
            usage,
            label: label.unwrap_or("unnamed").to_string(),
        }
    }
    /// Create a new empty GPU buffer with given usage flags
    pub fn new_empty(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &[],
            usage,
        });
        WgpuBuffer {
            buffer,
            size: 0,
            usage,
            label: label.unwrap_or("unnamed").to_string(),
        }
    }

    /// Update the buffer with new data via queue write
    pub fn write_data<T: bytemuck::Pod>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        data: &[T],
        offset: Option<u64>,
    ) {
        let bytes = bytemuck::cast_slice(data);
        let size = std::mem::size_of_val(data) as u64;
        if size > self.buffer.size() {
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: bytes,
                usage: self.usage,
            })
        }
        self.size = size as usize;
        queue.write_buffer(&self.buffer, offset.unwrap_or(0), bytes);
    }
}

pub type WgpuBufferCacheType = crate::HashCache<Arc<WgpuBuffer>>;
pub struct WgpuBufferManager {
    inner: WgpuBufferCacheType,
}

impl WgpuBufferManager {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl crate::CacheStorage<Arc<WgpuBuffer>> for WgpuBufferManager {
    fn get_resource(&self, key: &crate::CacheKey) -> Option<&Arc<WgpuBuffer>> {
        self.inner.get(key)
    }
    fn contains_resource(&self, key: &crate::CacheKey) -> bool {
        self.inner.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut Arc<WgpuBuffer>> {
        self.inner.get_mut(key)
    }
    fn get_or_create<F>(
        &mut self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> Result<Arc<WgpuBuffer>, EngineError>
    where
        F: FnOnce() -> Result<Arc<WgpuBuffer>, EngineError>,
    {
        self.inner.get_or_create(key, create_fn)
    }
    fn insert_resource(&mut self, key: crate::CacheKey, resource: Arc<WgpuBuffer>) {
        self.inner.insert(key, resource);
    }
    fn remove_resource(&mut self, key: &crate::CacheKey) -> Option<Arc<WgpuBuffer>> {
        self.inner.remove(key)
    }

    fn all<'a>(&'a self) -> impl Iterator<Item = &'a Arc<WgpuBuffer>>
    where
        WgpuBuffer: 'a,
    {
        self.inner.values()
    }
}
