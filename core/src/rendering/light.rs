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
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: crate::WgpuBuffer,
}

impl Light {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = LightUniform::LAYOUT;
    pub fn new(
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: cgmath::Vector3<f32>,
        color: cgmath::Vector3<f32>,
    ) -> Result<Self, crate::EngineError> {
        let bind_group_layout = crate::BindGroupLayouts::light();
        let uniform_buffer = WgpuBuffer::from_data(
            queue,
            device,
            &[LightUniform::new()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
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
