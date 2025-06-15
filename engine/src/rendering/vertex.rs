use std::hash::Hash;
use std::hash::Hasher;

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
    pub model: [[f32; 4]; 4], //  0–63   | @location(5..8)
    pub color: [f32; 3],      // 64–75   | @location(9)
    pub _pad0: f32,           // 76–79
    pub uv_offset: [f32; 2],  // 80–87   | @location(11)
    pub _pad1: [f32; 2],      // 88–95
    pub normal: [f32; 3],     // 96–107  | @location(12)
    pub _pad2: f32,           // 108–111
    pub tangent: [f32; 3],    // 112–123  | @location(13)
    pub _pad3: f32,           // 124–127
    pub material_id: u32,     // 128–131  | @location(14)
    pub _pad4: [f32; 3],      // 132–143
}
impl VertexInstance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<VertexInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // model matrix (mat4x4) → locations 5..8
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
            // color → location 9
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x3,
            },
            // uv_offset → location 10
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 10,
                format: wgpu::VertexFormat::Float32x2,
            },
            // normal → location 11
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32x3,
            },
            // tangent → location 12
            wgpu::VertexAttribute {
                offset: 112,
                shader_location: 12,
                format: wgpu::VertexFormat::Float32x3,
            },
            // material_id → location 13
            wgpu::VertexAttribute {
                offset: 128,
                shader_location: 13,
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
impl Hash for VertexInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for row in &self.model {
            for val in row {
                val.to_bits().hash(state);
            }
        }

        for val in &self.color {
            val.to_bits().hash(state);
        }

        for val in &self.uv_offset {
            val.to_bits().hash(state);
        }

        for val in &self.normal {
            val.to_bits().hash(state);
        }

        for val in &self.tangent {
            val.to_bits().hash(state);
        }

        self.material_id.hash(state);
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2d {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub texture_index: i32,
    _pad: [f32; 3], // Pad to 32 bytes for alignment
}

impl Vertex2d {
    pub fn new(
        position: [f32; 2],
        tex_coords: [f32; 2],
        color: [f32; 4],
        texture_index: i32,
    ) -> Self {
        Self {
            position,
            tex_coords,
            color,
            texture_index,
            _pad: [0.0; 3],
        }
    }
}

impl Vertex2d {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex2d>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 8,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: wgpu::VertexFormat::Sint32,
            },
        ],
    };
}

#[derive(Debug, Clone)]
pub struct OwnedVertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl OwnedVertexBufferLayout {
    pub fn convert_layouts(layouts: &[wgpu::VertexBufferLayout]) -> Vec<OwnedVertexBufferLayout> {
        layouts
            .iter()
            .map(|l| OwnedVertexBufferLayout {
                array_stride: l.array_stride,
                step_mode: l.step_mode,
                attributes: l.attributes.to_vec(),
            })
            .collect()
    }
    pub fn reconstruct_layouts(
        layouts: &[OwnedVertexBufferLayout],
    ) -> Vec<wgpu::VertexBufferLayout> {
        layouts
            .iter()
            .map(|l| wgpu::VertexBufferLayout {
                array_stride: l.array_stride,
                step_mode: l.step_mode,
                attributes: &l.attributes,
            })
            .collect()
    }
}
