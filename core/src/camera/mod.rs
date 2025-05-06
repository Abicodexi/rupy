pub mod controller;
pub use controller::*;

pub mod frustum;
pub use frustum::*;

pub mod uniform;
pub use uniform::*;

#[derive(Debug)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: cgmath::Deg<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub uniform: uniform::CameraUniform,
    pub frustum: Frustum,
    pub bind_group: crate::CacheKey,
    pub uniform_cache_key: crate::CacheKey,
}

impl Camera {
    pub fn update(&mut self, controller: &mut CameraController) {
        controller.update(self, 1.0 / 60.0);
        let vp = self.build_view_projection_matrix();
        self.frustum = self.frustum(vp.0);
        self.uniform.update(vp);
    }
    pub fn build_view_projection_matrix(
        &self,
    ) -> (
        cgmath::Matrix4<f32>,
        cgmath::Matrix4<f32>,
        cgmath::Matrix4<f32>,
    ) {
        use cgmath::perspective;
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = cgmath::SquareMatrix::invert(&view).unwrap();
        let inv_proj = cgmath::SquareMatrix::invert(&proj).unwrap();
        (proj * view, inv_proj, inv_view)
    }
    pub fn frustum(&self, vp: cgmath::Matrix4<f32>) -> Frustum {
        Frustum::from_matrix(vp)
    }
}
