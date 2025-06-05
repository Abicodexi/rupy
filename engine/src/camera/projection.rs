use glam::Mat4;

use super::Camera;

/// The three camera‐projection modes we support.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Projection {
    FirstPerson,
    ThirdPerson,
    Orthographic,
}

impl Projection {
    /// Given a `Camera`, return the appropriate projection‐matrix.
    pub fn projection_matrix(&self, camera: &Camera) -> Mat4 {
        match self {
            Projection::FirstPerson | Projection::ThirdPerson => {
                Mat4::perspective_rh_gl(camera.fovy, camera.aspect, camera.znear, camera.zfar)
            }
            Projection::Orthographic => {
                // Centered orthographic around the camera’s aspect ratio:
                // We’ll take “half‐height” = zfar/2, so the camera sees from +zfar/2 down to -zfar/2.
                let half_h = camera.zfar / 2.0;
                let half_w = half_h * camera.aspect;
                Mat4::orthographic_rh_gl(
                    -half_w,
                    half_w,
                    -half_h,
                    half_h,
                    camera.znear,
                    camera.zfar,
                )
            }
        }
    }
}
