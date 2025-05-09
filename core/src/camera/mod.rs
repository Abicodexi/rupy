pub mod controller;
use cgmath::EuclideanSpace;
pub use controller::*;

pub mod frustum;
pub use frustum::*;

pub mod uniform;
pub use uniform::*;

use crate::WgpuBuffer;

#[derive(Debug)]
pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: cgmath::Deg<f32>,
    znear: f32,
    zfar: f32,
    bind_group: wgpu::BindGroup,
    uniform_buffer: WgpuBuffer,
}

impl Camera {
    pub const UNIFORM_BUFFER_BINDING: crate::BindGroupBindingType = crate::BindGroupBindingType {
        binding: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                crate::camera::uniform::CameraUniform,
            >() as u64),
        },
    };
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let uniform_buffer = WgpuBuffer::from_data(
            device,
            &[CameraUniform::default()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Some("camera uniform buffer"),
        );
        let bind_group = crate::BindGroup::camera(device, &uniform_buffer);
        Camera {
            eye: cgmath::Point3::new(0.0, 0.0, 5.0),
            target: cgmath::Point3::new(0.0, 0.0, -5.0),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: cgmath::Deg(45.0),
            znear: 0.1,
            zfar: 100.0,
            bind_group,
            uniform_buffer,
        }
    }

    pub fn buffer(&self) -> &crate::WgpuBuffer {
        &self.uniform_buffer
    }
    pub fn upload(&mut self, queue: &wgpu::Queue) {
        self.uniform_buffer
            .write_data(queue, &[self.uniform()], None);
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn eye(&self) -> &cgmath::Point3<f32> {
        &self.eye
    }
    pub fn target(&self) -> &cgmath::Point3<f32> {
        &self.target
    }
    pub fn up(&self) -> &cgmath::Vector3<f32> {
        &self.up
    }
    pub fn fovy(&self) -> &cgmath::Deg<f32> {
        &self.fovy
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
        uniform.update(self.view_projection_matrix(), self.eye.to_vec());
        uniform
    }
    pub fn buffer_line(
        &mut self,
        line_ending: &glyphon::cosmic_text::LineEnding,
        attrs_list: &glyphon::AttrsList,
        shaping: &glyphon::Shaping,
    ) -> (glyphon::BufferLine, glyphon::BufferLine) {
        (
            glyphon::BufferLine::new(
                format!(
                    "Eye: x: {:.2} y: {:.2} z: {:.2}",
                    self.eye().x,
                    self.eye().y,
                    self.eye().z
                ),
                *line_ending,
                attrs_list.clone(),
                *shaping,
            ),
            glyphon::BufferLine::new(
                format!(
                    "Target: x: {:.2} y: {:.2} z: {:.2}",
                    self.target().x,
                    self.target().y,
                    self.target().z
                ),
                *line_ending,
                attrs_list.clone(),
                *shaping,
            ),
        )
    }
}
