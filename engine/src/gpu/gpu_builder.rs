use crate::GPU;

pub struct GpuBuilder<'a> {
    backends: wgpu::Backends,
    power_preference: wgpu::PowerPreference,
    compatible_surface: Option<wgpu::Surface<'a>>,
    required_features: wgpu::Features,
    required_limits: wgpu::Limits,
}

impl<'a> GpuBuilder<'a> {
    pub fn new() -> Self {
        Self {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            required_features: wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                | wgpu::Features::BGRA8UNORM_STORAGE,
            required_limits: wgpu::Limits::downlevel_defaults(),
        }
    }

    pub fn backends(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    pub fn power_preference(mut self, pref: wgpu::PowerPreference) -> Self {
        self.power_preference = pref;
        self
    }

    pub fn compatible_surface(mut self, surface: wgpu::Surface<'a>) -> Self {
        self.compatible_surface = Some(surface);
        self
    }

    pub fn features(mut self, features: wgpu::Features) -> Self {
        self.required_features = features;
        self
    }

    pub fn limits(mut self, limits: wgpu::Limits) -> Self {
        self.required_limits = limits;
        self
    }

    pub fn build(self) -> GPU {
        let instance = std::sync::Arc::new(wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: self.backends,
            flags: wgpu::InstanceFlags::empty(),
            ..Default::default()
        }));

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: self.power_preference,
            compatible_surface: self.compatible_surface.as_ref(),
            force_fallback_adapter: false,
        }))
        .expect("Failed to get adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: self.required_features,
                required_limits: self.required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to request device");

        GPU {
            instance,
            adapter: adapter.into(),
            device: device.into(),
            queue: queue.into(),
        }
    }
}

impl<'a> Default for GpuBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
