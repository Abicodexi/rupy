use wgpu_macros::VertexLayout;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
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
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
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
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
    pub row2: [f32; 4],
    pub row3: [f32; 4],
}

impl InstanceData {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
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
