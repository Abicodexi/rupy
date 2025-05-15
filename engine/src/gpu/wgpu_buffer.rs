use crate::log_info;

/// Wrapper around WGPU buffers
#[derive(Debug)]
pub struct WgpuBuffer {
    buffer: wgpu::Buffer,
    size: usize,
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
        }
    }
    pub fn get(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn size(&self) -> usize {
        self.size
    }
    /// Create a new empty GPU buffer with given usage flags

    pub fn new_empty(device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self {
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

pub type WgpuBufferCacheType = crate::HashCache<WgpuBuffer>;
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

impl crate::CacheStorage<WgpuBuffer> for WgpuBufferManager {
    fn get(&self, key: &crate::CacheKey) -> Option<&WgpuBuffer> {
        self.inner.get(key)
    }
    fn contains(&self, key: &crate::CacheKey) -> bool {
        self.inner.contains_key(key)
    }
    fn get_mut(&mut self, key: &crate::CacheKey) -> Option<&mut WgpuBuffer> {
        self.inner.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: crate::CacheKey, create_fn: F) -> &mut WgpuBuffer
    where
        F: FnOnce() -> WgpuBuffer,
    {
        self.inner.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: crate::CacheKey, resource: WgpuBuffer) {
        self.inner.insert(key, resource);
    }
    fn remove(&mut self, key: &crate::CacheKey) {
        self.inner.remove(key);
    }
}
