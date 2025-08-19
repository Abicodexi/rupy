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
        let size = (std::mem::size_of::<T>() * data.len()) as u64;
        let contents = bytemuck::cast_slice(data);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents,
            usage,
        });
        WgpuBuffer {
            buffer,
            size: size as usize,
            usage,
            label: label.unwrap_or("unnamed").to_string(),
        }
    }

    #[inline]
    pub fn get(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    #[inline]
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

    /// Create a new empty GPU buffer with given usage flags (size = 0).
    pub fn new_empty(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: 0,
            usage,
            mapped_at_creation: false,
        });
        WgpuBuffer {
            buffer,
            size: 0,
            usage,
            label: label.unwrap_or("unnamed").to_string(),
        }
    }

    /// Update the buffer with new data via queue write.
    /// If the new data is larger than current capacity, the buffer is re-created.
    pub fn write_data<T: bytemuck::Pod>(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        data: &[T],
        offset: Option<u64>,
    ) {
        let bytes = bytemuck::cast_slice(data);
        let size = bytes.len() as u64;

        if size > self.buffer.size() {
            // Reallocate to fit new contents.
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: bytes,
                usage: self.usage,
            });
        } else {
            queue.write_buffer(&self.buffer, offset.unwrap_or(0), bytes);
        }

        self.size = size as usize;
    }
}

