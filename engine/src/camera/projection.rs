use crate::camera::{CameraModel, CameraTransform};

use glam::{Mat4, Quat, Vec3};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Projection {
    FirstPerson,
    ThirdPerson,
    Orthographic,
}

impl Projection {
    pub fn perspective(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
        Mat4::perspective_rh_gl(fov_y_radians, aspect_ratio, z_near, z_far)
    }
    pub fn orthographic(screen_w: f32, screen_h: f32) -> Mat4 {
        glam::Mat4::orthographic_rh_gl(0.0, screen_w, screen_h, 0.0, -1.0, 1.0)
    }
    pub fn matrix(
        &self,
        fov_y_radians: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> Mat4 {
        match self {
            Projection::FirstPerson | Projection::ThirdPerson => {
                Projection::perspective(fov_y_radians, aspect_ratio, z_near, z_far)
            }
            Projection::Orthographic => Projection::orthographic(screen_w, screen_h),
        }
    }
    pub fn apply_to_transform(
        &self,
        transform: &mut CameraTransform,
        position: Vec3,
        yaw: f32,
        pitch: f32,
        model: &CameraModel,
    ) {
        match self {
            Projection::FirstPerson => {
                transform.set_eye(position + Vec3::Y * 1.6);
                transform.set_target(
                    transform.eye
                        + Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, 0.0) * -Vec3::Z,
                );
                transform.set_up(Vec3::Y);
            }

            Projection::ThirdPerson => {
                transform.set_eye(
                    position
                        + Quat::from_euler(glam::EulerRot::YXZ, yaw, 0.0, 0.0)
                            * Vec3::Z
                            * model.distance()
                        + Vec3::Y * model.height(),
                );
                transform.set_target(position + Vec3::Y * model.target_height());
                transform.set_up(Vec3::Y);
            }

            Projection::Orthographic => {
                transform.set_eye(position + Vec3::Y * model.distance().max(10.0));
                transform.set_target(position);
                transform.set_up(Vec3::Y);
            }
        }
    }
    pub fn process(&self, event: &WindowEvent) -> Option<Self> {
        match &event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() && event.repeat == false {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyM) => return Some(self.next()),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        None
    }
    pub fn next(&self) -> Projection {
        match self {
            Projection::FirstPerson => Projection::ThirdPerson,
            Projection::ThirdPerson => Projection::Orthographic,
            Projection::Orthographic => Projection::FirstPerson,
        }
    }
}
