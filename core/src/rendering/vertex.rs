#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, wgpu_macros::VertexLayout)]
pub struct VertexColor {
    pub(crate) position: [f32; 3], // @location(0)
    pub(crate) color: [f32; 3],    // @location(1)
}

impl Default for VertexColor {
    fn default() -> Self {
        Self {
            position: Default::default(),
            color: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, wgpu_macros::VertexLayout)]
pub struct VertexTexture {
    pub position: [f32; 3],   // @location(0)
    pub color: [f32; 3],      // @location(1)
    pub tex_coords: [f32; 2], // @location(2)
}

impl Default for VertexTexture {
    fn default() -> Self {
        Self {
            position: Default::default(),
            color: Default::default(),
            tex_coords: Default::default(),
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, wgpu_macros::VertexLayout)]
pub struct VertexNormal {
    pub position: [f32; 3],   // @location(0)
    pub tex_coords: [f32; 2], // @location(1)
    pub normal: [f32; 3],     // @location(2)
    pub tangent: [f32; 3],    // @location(3)
    pub bitangent: [f32; 3],  // @location(4)
}
impl Default for VertexNormal {
    fn default() -> Self {
        Self {
            position: Default::default(),
            tex_coords: Default::default(),
            normal: Default::default(),
            tangent: Default::default(),
            bitangent: Default::default(),
        }
    }
}

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    const LAYOUT: wgpu::VertexBufferLayout<'static>;
}

impl Vertex for VertexColor {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = VertexColor::LAYOUT;
}
impl Vertex for VertexTexture {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = VertexTexture::LAYOUT;
}
impl Vertex for VertexNormal {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = VertexNormal::LAYOUT;
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexTextureInstance {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
    pub row2: [f32; 4],
    pub row3: [f32; 4],
}

impl VertexTextureInstance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<VertexTextureInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 6,
            },
        ],
    };
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexNormalInstance {
    // your 4×4 model matrix:
    pub row0: [f32; 4], // offset  0
    pub row1: [f32; 4], // offset 16
    pub row2: [f32; 4], // offset 32
    pub row3: [f32; 4], // offset 48

    // per‐instance normal/tangent/bitangent — each 3 floats + 1 padding
    pub normal: [f32; 3],    // offset 64
    pub _pad0: f32,          // offset 76
    pub tangent: [f32; 3],   // offset 80
    pub _pad1: f32,          // offset 92
    pub bitangent: [f32; 3], // offset 96
    pub _pad2: f32,          // offset108
}

// 64 bytes for the matrix + 3×16 bytes for N/T/B = 112 bytes total
impl VertexNormalInstance {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<VertexNormalInstance>() as wgpu::BufferAddress, // 112
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // matrix rows @ locs 5–8
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
            // normal @ loc 9
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x3,
            },
            // tangent @ loc 10
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 10,
                format: wgpu::VertexFormat::Float32x3,
            },
            // bitangent @ loc 11
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 11,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    };
}
