use cgmath::{Angle, Deg, InnerSpace, Vector3, Zero};
use winit::{
    event::{ElementState, MouseScrollDelta, WindowEvent},
    keyboard::KeyCode,
};

use super::Camera;

#[derive(Debug)]
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub pitch: f32,
    pub yaw: f32,
    last_mouse: Option<(f32, f32)>,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            back: false,
            left: false,
            right: false,
            pitch: 0.0,
            yaw: -90.0,
            last_mouse: None,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyW) => {
                        self.forward = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => {
                        self.back = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => {
                        self.left = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyD) => {
                        self.right = down;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);
                if let Some((lx, ly)) = self.last_mouse {
                    let dx = (x - lx) * self.sensitivity;
                    let dy = (y - ly) * self.sensitivity;
                    self.yaw += dx;
                    self.pitch = (self.pitch + dy).clamp(-89.0, 89.0);
                }
                self.last_mouse = Some((x, y));
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, _scroll) = delta {}
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        let (yaw_r, pitch_r): (cgmath::Deg<f32>, cgmath::Deg<f32>) =
            (Deg(self.yaw).into(), Deg(self.pitch).into());
        let front = Vector3 {
            x: yaw_r.cos() * pitch_r.cos(),
            y: pitch_r.sin(),
            z: yaw_r.sin() * pitch_r.cos(),
        }
        .normalize();

        let right = front.cross(camera.up).normalize();
        let mut displacement = Vector3::zero();
        if self.forward {
            displacement += front;
        }
        if self.back {
            displacement -= front;
        }
        if self.right {
            displacement += right;
        }
        if self.left {
            displacement -= right;
        }
        if displacement.magnitude2() > 0.0 {
            let disp = displacement.normalize() * self.speed * dt;
            camera.eye += disp;
        }

        camera.target = camera.eye + front;
    }
}
