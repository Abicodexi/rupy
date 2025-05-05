use cgmath::InnerSpace as _;

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

    pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == winit::event::ElementState::Pressed;
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => {
                        self.forward = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => {
                        self.back = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => {
                        self.left = down;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => {
                        self.right = down;
                        true
                    }
                    _ => false,
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
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
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(_, _scroll) = delta {}
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, camera: &mut super::Camera, dt: f32) {
        let (yaw_r, pitch_r): (cgmath::Deg<f32>, cgmath::Deg<f32>) =
            (cgmath::Deg(self.yaw).into(), cgmath::Deg(self.pitch).into());
        let front = cgmath::Vector3 {
            x: cgmath::Angle::cos(yaw_r) * cgmath::Angle::cos(pitch_r),
            y: cgmath::Angle::sin(pitch_r),
            z: cgmath::Angle::sin(yaw_r) * cgmath::Angle::cos(pitch_r),
        }
        .normalize();

        let right = front.cross(camera.up).normalize();
        let mut displacement = <cgmath::Vector3<f32> as cgmath::Zero>::zero();
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
