#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Vertex {
    pub position: [f32; 3],   // @location(0)
    pub color: [f32; 3],      // @location(1)
    pub tex_coords: [f32; 2], // @location(2)
    pub normal: [f32; 3],     // @location(3)
    pub tangent: [f32; 3],    // @location(4)
}
impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 44,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    };
}
pub trait VertexLayout: bytemuck::Pod + bytemuck::Zeroable {
    const LAYOUT: wgpu::VertexBufferLayout<'static>;
}

impl VertexLayout for Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = Vertex::LAYOUT;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct VertexInstance {
    pub model: [[f32; 4]; 4],  // 0–63   | @location(5..8)
    pub color: [f32; 3],       // 64–75  | @location(9)
    pub _pad0: f32,            // 76–79
    pub translation: [f32; 3], // 80–91  | @location(10)
    pub _pad1: f32,            // 92–95
    pub uv_offset: [f32; 2],   // 96–103 | @location(11)
    pub _pad2: [f32; 2],       // 104–111
    pub normal: [f32; 3],      // 112–123| @location(12)
    pub _pad3: f32,            // 124–127
    pub tangent: [f32; 3],     // 128–139| @location(13)
    pub _pad4: f32,            // 140–143
    pub material_id: u32,      // 144–147| @location(14)
    pub _pad5: [f32; 3],       // 148–159 (pad to 16 bytes)
}
impl VertexInstance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<VertexInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // model matrix (mat4x4) → locations 5–8
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
            // color → 9
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x3,
            },
            // translation → 10
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 10,
                format: wgpu::VertexFormat::Float32x3,
            },
            // uv_offset → 11
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32x2,
            },
            // normal → 12
            wgpu::VertexAttribute {
                offset: 112,
                shader_location: 12,
                format: wgpu::VertexFormat::Float32x3,
            },
            // tangent → 13
            wgpu::VertexAttribute {
                offset: 128,
                shader_location: 13,
                format: wgpu::VertexFormat::Float32x3,
            },
            // material_id → 14
            wgpu::VertexAttribute {
                offset: 144,
                shader_location: 14,
                format: wgpu::VertexFormat::Uint32,
            },
        ],
    };
    pub fn bytes(instances: &[VertexInstance]) -> Vec<u8> {
        let mut data = Vec::with_capacity(instances.len() * std::mem::size_of::<VertexInstance>());
        for inst in instances {
            data.extend_from_slice(bytemuck::bytes_of(inst));
        }
        data
    }
}
