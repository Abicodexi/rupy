use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
    _pad: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_view: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: Vec3::ZERO.to_array(),
            _pad: 0.0,
        }
    }

    pub fn update(&mut self, vp: Mat4, inv_proj: Mat4, inv_view: Mat4, position: Vec3) {
        self.view_proj = vp.to_cols_array_2d();
        self.inv_proj = inv_proj.to_cols_array_2d();
        self.inv_view = inv_view.to_cols_array_2d();
        self.view_pos = position.to_array();
    }
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OrthoUniform {
    pub proj: [[f32; 4]; 4],
}

impl OrthoUniform {
    pub fn new(width: f32, height: f32) -> Self {
        let ortho = glam::Mat4::orthographic_rh_gl(0.0, width, height, 0.0, -1.0, 1.0);
        Self {
            proj: ortho.to_cols_array_2d(),
        }
    }
}
