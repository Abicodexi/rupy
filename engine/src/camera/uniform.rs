use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    view_pos: [f32; 3],
    _pad: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_view: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: Vec3::ZERO.to_array(),
            _pad: 0.0,
        }
    }
    pub fn pos(&self) -> [f32; 3] {
        self.view_pos
    }
    pub fn update(
        &mut self,
        vp: (Mat4, Mat4, Mat4), // (view_proj, inv_proj, inv_view)
        view_pos: Vec3,
    ) {
        let (vp, inv_proj, inv_view) = vp;
        self.view_proj = vp.to_cols_array_2d();
        self.inv_proj = inv_proj.to_cols_array_2d();
        self.inv_view = inv_view.to_cols_array_2d();
        self.view_pos = view_pos.to_array();
    }
}
