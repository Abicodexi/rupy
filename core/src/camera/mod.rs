pub mod controller;
pub use controller::*;

pub mod frustum;
pub use frustum::*;

pub mod uniform;
pub use uniform::*;

use crate::WgpuBuffer;

#[derive(Debug)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: cgmath::Deg<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: WgpuBuffer,
}

impl Camera {
    pub fn new(queue: &wgpu::Queue, device: &wgpu::Device, aspect: f32) -> Self {
        let uniform = CameraUniform::default();
        let uniform_buffer = WgpuBuffer::from_data(
            queue,
            device,
            &[uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera uniform bind group"),
            layout: &crate::BindGroupLayouts::camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.get().as_entire_binding(),
            }],
        });
        Camera {
            eye: cgmath::Point3::new(0.0, 1.0, 2.0),
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: cgmath::Deg(45.0),
            znear: 0.1,
            zfar: 1000.0,
            bind_group,
            uniform_buffer,
        }
    }
    pub fn update(&mut self, controller: &mut CameraController) {
        controller.update(self, 1.0 / 60.0);
    }
    pub fn view_projection_matrix(
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
    pub fn frustum(&self) -> Frustum {
        let vp = self.view_projection_matrix();
        Frustum::from_matrix(vp.0)
    }
    pub fn uniform(&self) -> crate::camera::CameraUniform {
        let mut uniform = crate::camera::CameraUniform::new();
        uniform.update(self.view_projection_matrix());
        uniform
    }
}
