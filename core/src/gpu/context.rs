use std::sync::Arc;

use wgpu::{
    Adapter, Backends, Device, Features, Instance, InstanceDescriptor, InstanceFlags, Limits,
    MemoryHints, Queue, RequestAdapterOptions,
};

use crate::EngineError;

pub struct GpuContext {
    pub instance: Arc<Instance>,
    pub adapter: Arc<Adapter>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl GpuContext {
    pub async fn new() -> Result<Self, EngineError> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::default(),
            flags: InstanceFlags::empty(),
            backend_options: Default::default(),
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .expect("Request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Features::POLYGON_MODE_LINE | Features::POLYGON_MODE_POINT,
                    required_limits: Limits::downlevel_defaults(),
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(Self {
            instance: instance.into(),
            adapter: adapter.into(),
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
