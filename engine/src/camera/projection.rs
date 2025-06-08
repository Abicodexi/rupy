use super::Camera;
use glam::Mat4;

/// The three camera‐projection modes we support.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Projection {
    FirstPerson,
    ThirdPerson,
    Orthographic,
}

impl Projection {
    pub fn perspective(camera: &Camera) -> Mat4 {
        Mat4::perspective_rh_gl(camera.fovy, camera.aspect, camera.znear, camera.zfar)
    }
    pub fn orthographic(screen_w: f32, screen_h: f32) -> Mat4 {
        glam::Mat4::orthographic_rh_gl(0.0, screen_w, screen_h, 0.0, -1.0, 1.0)
    }
    /// Given a `Camera`, return the appropriate projection‐matrix.
    pub fn matrix(&self, camera: &Camera, screen_w: f32, screen_h: f32) -> Mat4 {
        match self {
            Projection::FirstPerson | Projection::ThirdPerson => Projection::perspective(camera),
            Projection::Orthographic => Projection::orthographic(screen_w, screen_h),
        }
    }
    pub fn next(&self) -> Projection {
        match self {
            Projection::FirstPerson => Projection::ThirdPerson,
            Projection::ThirdPerson => Projection::Orthographic,
            Projection::Orthographic => Projection::FirstPerson,
        }
    }
}
