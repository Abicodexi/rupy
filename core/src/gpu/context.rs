static GPU: std::sync::OnceLock<std::sync::Arc<std::sync::RwLock<GPU>>> =
    std::sync::OnceLock::new();

fn init_global_gpu() {
    let gpu = GPU::new();
    let arc_gpu = std::sync::Arc::new(std::sync::RwLock::new(gpu));
    GPU.set(arc_gpu)
        .expect("Global gpu was already initialized");
}

fn gpu() -> std::sync::Arc<std::sync::RwLock<GPU>> {
    GPU.get().expect("Global gpu is not initialized").clone()
}

#[derive(Debug)]
pub struct GPU {
    instance: std::sync::Arc<wgpu::Instance>,
    adapter: std::sync::Arc<wgpu::Adapter>,
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
}

impl GPU {
    pub fn get() -> std::sync::Arc<std::sync::RwLock<GPU>> {
        gpu()
    }
    pub fn init() {
        init_global_gpu();
    }
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::default(),
            flags: wgpu::InstanceFlags::empty(),
            backend_options: Default::default(),
        });

        let adapter = pollster::FutureExt::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .expect("Request adapter");

        let (device, queue) = pollster::FutureExt::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::POLYGON_MODE_POINT,
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to request device and/or queue from adapter");

        Self {
            instance: instance.into(),
            adapter: adapter.into(),
            device: device.into(),
            queue: queue.into(),
        }
    }

    pub fn instance(&self) -> &std::sync::Arc<wgpu::Instance> {
        &self.instance
    }

    pub fn adapter(&self) -> &std::sync::Arc<wgpu::Adapter> {
        &self.adapter
    }

    pub fn device(&self) -> &std::sync::Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &std::sync::Arc<wgpu::Queue> {
        &self.queue
    }
}
