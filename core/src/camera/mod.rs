pub mod controller;
pub mod frustum;
pub mod uniform;

use std::sync::Arc;

use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};
use controller::CameraController;
use frustum::Frustum;

use crate::CacheKey;

#[derive(Debug)]
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: Deg<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub uniform: uniform::CameraUniform,
    pub frustum: Frustum,
    pub bind_group: wgpu::BindGroup,
    pub uniform_cache_key: CacheKey,
}

impl Camera {
    pub fn update(&mut self, controller: &mut CameraController) {
        controller.update(self, 1.0 / 60.0);
        let vp = self.build_view_projection_matrix();
        self.frustum = self.frustum(vp.0);
        self.uniform.update(vp);
    }
    pub fn build_view_projection_matrix(&self) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = view.invert().unwrap();
        let inv_proj = proj.invert().unwrap();
        (proj * view, inv_proj, inv_view)
    }
    pub fn frustum(&self, vp: Matrix4<f32>) -> Frustum {
        Frustum::from_matrix(vp)
    }
}
