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
    pub uniform: uniform::CameraUniform,
    pub controller: controller::CameraController,
}

impl Camera {
    pub fn update(&mut self) {
        let camera: (Deg<f32>, Deg<f32>, Point3<f32>, Point3<f32>, Vector3<f32>) = (
            Deg(self.controller.yaw).into(),
            Deg(self.controller.pitch).into(),
            self.eye,
            self.target,
            self.up,
        );

        self.controller.update(camera, 1.0 / 60.0);
        let vp = self.build_view_projection_matrix();
        self.uniform.update(vp);
    }
    pub fn build_view_projection_matrix(&self) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        let inv_view = view.invert().unwrap();
        let inv_proj = proj.invert().unwrap();
        (proj * view, inv_proj, inv_view)
    }
}
