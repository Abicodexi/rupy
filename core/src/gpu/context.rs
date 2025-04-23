use std::sync::Arc;

use wgpu::{Adapter, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions};

use crate::EngineError;

pub struct GpuContext {
    pub instance: Instance,
    pub adapter: wgpu::Adapter,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl GpuContext {
    pub async fn new() -> Result<Self, EngineError> {
        let instance = Instance::new(InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .ok_or(EngineError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        Ok(Self {
            instance,
            adapter,
            device: device.into(),
            queue: queue.into(),
        })
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
