pub mod controller;
pub mod uniform;

use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};

#[derive(Debug)]
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: Deg<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = view.invert().unwrap();
        let inv_proj = proj.invert().unwrap();
        (proj * view, inv_proj, inv_view)
    }
}
