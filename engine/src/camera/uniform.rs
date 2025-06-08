use glam::{Mat4, Vec3};

//
// --------------
//  CAMERA UNIFORM
// --------------

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// (projection * view) matrix
    pub view_proj: [[f32; 4]; 4],
    /// inverse of the projection matrix
    pub inv_proj: [[f32; 4]; 4],
    /// inverse of the view matrix
    pub inv_view: [[f32; 4]; 4],
    /// camera world‐position (xyz); w is unused
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

    /// Overwrite this uniform given:
    ///   - view:    Mat4 look‐at matrix (camera→world)
    ///   - proj:    Mat4 projection matrix (perspective or ortho)
    ///   - cam_pos: camera’s world position
    pub fn update<P: Into<Mat4>>(&mut self, view: Mat4, proj: P, cam_pos: Vec3) {
        let proj_mat: Mat4 = proj.into();
        let vp = proj_mat * view;
        let inv_proj = proj_mat.inverse();
        let inv_view = view.inverse();

        self.view_proj = vp.to_cols_array_2d();
        self.inv_proj = inv_proj.to_cols_array_2d();
        self.inv_view = inv_view.to_cols_array_2d();
        self.view_pos = cam_pos.to_array();
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
