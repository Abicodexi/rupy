#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, wgpu_macros::VertexLayout)]
pub struct UnifiedVertex {
    /// @location(0)
    pub position: [f32; 3], // offset  0
    /// @location(1)
    pub color: [f32; 3], // offset 12
    /// @location(2)
    pub tex_coords: [f32; 2], // offset 24
    /// @location(3)
    pub normal: [f32; 3], // offset 32
    /// @location(4)
    pub tangent: [f32; 3], // offset 44
    /// @location(5)
    pub bitangent: [f32; 3], // offset 56
}
pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    const LAYOUT: wgpu::VertexBufferLayout<'static>;
}

impl Vertex for UnifiedVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = UnifiedVertex::LAYOUT;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct UnifiedVertexInstance {
    /// 4×4 model matrix
    pub model: [[f32; 4]; 4], //  0–63

    /// Instance color (RGB)
    pub color: [f32; 3], // 64–75
    pub _pad0: f32, // 76–79

    /// Instance translation (XYZ)
    pub translation: [f32; 3], // 80–91
    pub _pad1: f32, // 92–95

    /// UV offset (U, V)
    pub uv_offset: [f32; 2], // 96–103
    pub _pad2: [f32; 2], // 104–111

    /// Instance normal
    pub normal: [f32; 3], // 112–123
    pub _pad3: f32, // 124–127

    /// Instance tangent
    pub tangent: [f32; 3], // 128–139
    pub _pad4: f32, // 140–143

    /// Instance bitangent
    pub bitangent: [f32; 3], // 144–155
    pub _pad5: f32, // 156–159
}
impl UnifiedVertexInstance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<UnifiedVertexInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // model matrix → locations 6–9
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x4,
            },
            // color → location 10
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 10,
                format: wgpu::VertexFormat::Float32x3,
            },
            // translation → location 11
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32x3,
            },
            // uv_offset → location 12
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 12,
                format: wgpu::VertexFormat::Float32x2,
            },
            // normal → location 13
            wgpu::VertexAttribute {
                offset: 112,
                shader_location: 13,
                format: wgpu::VertexFormat::Float32x3,
            },
            // tangent → location 14
            wgpu::VertexAttribute {
                offset: 128,
                shader_location: 14,
                format: wgpu::VertexFormat::Float32x3,
            },
            // bitangent → location 15
            wgpu::VertexAttribute {
                offset: 144,
                shader_location: 15,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    };
    pub fn to_data(instances: Vec<UnifiedVertexInstance>) -> (u32, Vec<u8>) {
        let count = instances.len() as u32;
        let mut data = Vec::with_capacity(
            instances.len() * std::mem::size_of::<crate::UnifiedVertexInstance>(),
        );

        for inst in instances {
            data.extend_from_slice(bytemuck::bytes_of(&inst));
        }
        (count, data)
    }
}
