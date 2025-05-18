use crate::WgpuBuffer;

#[repr(C)]
#[derive(
    Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, wgpu_macros::VertexLayout, Default,
)]
pub struct LightUniform {
    position: [f32; 3],
    _pad0: f32,
    color: [f32; 3],
    _pad1: f32,
}

impl LightUniform {
    pub fn new() -> Self {
        Self {
            position: [5.0, 5.0, 1.0],
            _pad0: 1.0,
            color: [1.0, 1.0, 1.0],
            _pad1: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct Light {
    position: cgmath::Vector3<f32>,
    color: cgmath::Vector3<f32>,
    bind_group: wgpu::BindGroup,
    uniform_buffer: crate::WgpuBuffer,
}

impl Light {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = LightUniform::LAYOUT;
    pub const CENTER: cgmath::Vector3<f32> = cgmath::Vector3::new(1.0, 100.0, 1.0);
    pub const RADIUS: f32 = 360.0;
    pub const BUFFER_BINDING: crate::BindGroupBindingType = crate::BindGroupBindingType {
        binding: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(
                std::mem::size_of::<crate::LightUniform>() as u64
            ),
        },
    };

    pub fn new(device: &wgpu::Device) -> Result<Self, crate::EngineError> {
        let position: cgmath::Vector3<f32> = Self::CENTER.into();
        let color: cgmath::Vector3<f32> = [1.0, 1.0, 1.0].into();
        let bind_group_layout = crate::BindGroupLayouts::light();
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[LightUniform::new()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("light uniform buffer"),
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
            label: None,
        });

        Ok(Light {
            position,
            color,
            bind_group,
            uniform_buffer,
        })
    }

    pub fn set_position(&mut self, new_position: cgmath::Vector3<f32>) {
        self.position = new_position;
    }

    pub fn orbit(&mut self, time_s: f64) {
        let angle = time_s; // 1 radian/s;
        let (sin, cos) = angle.sin_cos();

        self.position.x = Light::CENTER.x + Light::RADIUS * cos as f32;
        self.position.z = Light::CENTER.z + Light::RADIUS * sin as f32;
    }
    pub fn buffer(&self) -> &crate::WgpuBuffer {
        &self.uniform_buffer
    }
    pub fn upload(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.uniform_buffer
            .write_data(queue, device, &[self.uniform()], None);
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn uniform(&self) -> LightUniform {
        let position: [f32; 3] = [self.position.x, self.position.y, self.position.z];
        let color: [f32; 3] = [self.color.x, self.color.y, self.color.z];
        LightUniform {
            position,
            _pad0: 1.0,
            color,
            _pad1: 1.0,
        }
    }
}
