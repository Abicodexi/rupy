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
    pub(crate) position: [f32; 3],   // @location(0)
    pub(crate) color: [f32; 3],      // @location(1)
    pub(crate) tex_coords: [f32; 2], // @location(2)
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
    pub(crate) position: [f32; 3],   // @location(0)
    pub(crate) tex_coords: [f32; 2], // @location(1)
    pub(crate) normal: [f32; 3],     // @location(2)
    pub(crate) tangent: [f32; 3],    // @location(3)
    pub(crate) bitangent: [f32; 3],  // @location(4)
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
