pub struct GpuContext {
    pub instance: std::sync::Arc<wgpu::Instance>,
    pub adapter: std::sync::Arc<wgpu::Adapter>,
    pub device: std::sync::Arc<wgpu::Device>,
    pub queue: std::sync::Arc<wgpu::Queue>,
}

impl GpuContext {
    pub async fn new() -> Result<Self, crate::EngineError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::default(),
            flags: wgpu::InstanceFlags::empty(),
            backend_options: Default::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::POLYGON_MODE_POINT,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
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

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
